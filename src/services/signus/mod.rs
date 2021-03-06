mod ed25519;
pub mod types;

extern crate serde_json;

use self::serde_json::Value;

use self::ed25519::ED25519Signus;
use self::types::{
    MyDidInfo,
    MyDid,
    TheirDidInfo,
    TheirDid
};
use utils::crypto::base58::Base58;
use utils::crypto::signature_serializer::serialize_signature;

use errors::common::CommonError;
use errors::signus::SignusError;

use std::collections::HashMap;
use std::error::Error;
use std::str;

const DEFAULT_CRYPTO_TYPE: &'static str = "ed25519";

trait CryptoType {
    fn encrypt(&self, private_key: &[u8], public_key: &[u8], doc: &[u8], nonce: &[u8]) -> Vec<u8>;
    fn decrypt(&self, private_key: &[u8], public_key: &[u8], doc: &[u8], nonce: &[u8]) -> Result<Vec<u8>, CommonError>;
    fn gen_nonce(&self) -> Vec<u8>;
    fn create_key_pair_for_signature(&self, seed: Option<&[u8]>) -> Result<(Vec<u8>, Vec<u8>), CommonError>;
    fn sign(&self, private_key: &[u8], doc: &[u8]) -> Result<Vec<u8>, CommonError>;
    fn verify(&self, public_key: &[u8], doc: &[u8], signature: &[u8]) -> Result<bool, CommonError>;
    fn verkey_to_public_key(&self, vk: &[u8]) -> Result<Vec<u8>, CommonError>;
    fn signkey_to_private_key(&self, sk: &[u8]) -> Result<Vec<u8>, CommonError>;
}

pub struct SignusService {
    crypto_types: HashMap<&'static str, Box<CryptoType>>
}

impl SignusService {
    pub fn new() -> SignusService {
        let mut crypto_types: HashMap<&str, Box<CryptoType>> = HashMap::new();
        crypto_types.insert(DEFAULT_CRYPTO_TYPE, Box::new(ED25519Signus::new()));

        SignusService {
            crypto_types: crypto_types
        }
    }

    pub fn create_my_did(&self, my_did_info: &MyDidInfo) -> Result<MyDid, SignusError> {
        let xtype = my_did_info.crypto_type.clone().unwrap_or(DEFAULT_CRYPTO_TYPE.to_string());

        if !self.crypto_types.contains_key(&xtype.as_str()) {
            return Err(
                SignusError::UnknownCryptoError(
                    format!("MyDidInfo info contains unknown crypto: {}", xtype)));
        }

        let signus = self.crypto_types.get(&xtype.as_str()).unwrap();

        let seed = my_did_info.seed.as_ref().map(String::as_bytes);
        let (ver_key, sign_key) = signus.create_key_pair_for_signature(seed)?;

        let public_key = signus.verkey_to_public_key(&ver_key)?;
        let secret_key = signus.signkey_to_private_key(&sign_key)?;

        let did = match my_did_info.did {
            Some(ref did) => Base58::decode(did)?,
            _ if my_did_info.cid == Some(true) => ver_key.clone(),
            _ => ver_key[0..16].to_vec()
        };

        let my_did = MyDid::new(Base58::encode(&did),
                                xtype.clone(),
                                Base58::encode(&public_key),
                                Base58::encode(&secret_key),
                                Base58::encode(&ver_key),
                                Base58::encode(&sign_key));

        Ok(my_did)
    }

    pub fn create_their_did(&self, their_did_info: &TheirDidInfo) -> Result<TheirDid, SignusError> {
        let xtype = their_did_info.crypto_type.clone().unwrap_or(DEFAULT_CRYPTO_TYPE.to_string());

        if !self.crypto_types.contains_key(&xtype.as_str()) {
            return Err(
                SignusError::UnknownCryptoError(
                    format!("TheirDidInfo info contains unknown crypto: {}", xtype)));
        }

        let signus = self.crypto_types.get(&xtype.as_str()).unwrap();

        // Check did is correct Base58
        Base58::decode(&their_did_info.did)?;

        let (verkey, pk) = match their_did_info.verkey {
            Some(ref verkey) => (
                Some(verkey.clone()),
                Some(Base58::encode(&signus.verkey_to_public_key(&Base58::decode(verkey)?)?))),
            None => (None, None)
        };

        let their_did = TheirDid::new(their_did_info.did.clone(),
                                      xtype.clone(),
                                      verkey,
                                      pk,
                                      their_did_info.endpoint.as_ref().cloned());
        Ok(their_did)
    }

    pub fn sign(&self, my_did: &MyDid, doc: &str) -> Result<String, SignusError> {
        if !self.crypto_types.contains_key(&my_did.crypto_type.as_str()) {
            return Err(
                SignusError::UnknownCryptoError(
                    format!("Trying to sign message with unknown crypto: {}", my_did.crypto_type)));
        }

        let signus = self.crypto_types.get(&my_did.crypto_type.as_str()).unwrap();

        let sign_key = Base58::decode(&my_did.signkey)?;
        let mut msg: Value = serde_json::from_str(doc)
            .map_err(|err|
                SignusError::CommonError(
                    CommonError::InvalidStructure(format!("Message is invalid json: {}", err.description()))))?;

        if !msg.is_object() {
            return Err(SignusError::CommonError(
                CommonError::InvalidStructure(format!("Message is invalid json: {}", msg))))
        }

        let signature = serialize_signature(msg.clone())?;
        let signature = signus.sign(&sign_key, signature.as_bytes())?;
        let signature = Base58::encode(&signature);
        msg["signature"] = Value::String(signature);
        let signed_msg: String = serde_json::to_string(&msg)
            .map_err(|err|
                SignusError::CommonError(
                    CommonError::InvalidState(format!("Can't serialize message after signing: {}", err.description()))))?;
        Ok(signed_msg)
    }

    pub fn verify(&self, their_did: &TheirDid, signed_msg: &str) -> Result<bool, SignusError> {
        if !self.crypto_types.contains_key(their_did.crypto_type.as_str()) {
            return Err(SignusError::UnknownCryptoError(format!("Trying to verify message with unknown crypto: {}", their_did.crypto_type)));
        }

        let signus = self.crypto_types.get(their_did.crypto_type.as_str()).unwrap();

        let verkey = match their_did.verkey {
            Some(ref verkey) => Base58::decode(&verkey)?,
            None => return Err(SignusError::CommonError(CommonError::InvalidStructure(format!("TheirDid doesn't contain verkey: {}", their_did.did))))
        };

        let signed_msg: Value = serde_json::from_str(signed_msg)
            .map_err(|err|
                SignusError::CommonError(
                    CommonError::InvalidStructure(format!("Message is invalid json: {}", err.description()))))?;

        if !signed_msg.is_object() {
            return Err(SignusError::CommonError(
                CommonError::InvalidStructure(format!("Message is invalid json: {}", signed_msg))))
        }

        // TODO: FIXME: This code seem unreliable and hard to understand
        if let Value::String(ref signature) = signed_msg["signature"] {
            let signature = Base58::decode(signature)?;
            let mut message: Value = Value::Object(serde_json::map::Map::new());
            for key in signed_msg.as_object().unwrap().keys() {
                if key != "signature" {
                    message[key] = signed_msg[key].clone();
                }
            }
            Ok(signus.verify(&verkey, &serialize_signature(message)?.as_bytes(), &signature)?)
        } else {
            return Err(SignusError::CommonError(CommonError::InvalidStructure(format!("No signature field in message json"))));
        }
    }

    pub fn encrypt(&self, my_did: &MyDid, their_did: &TheirDid, doc: &str) -> Result<(String, String), SignusError> {
        if !self.crypto_types.contains_key(&my_did.crypto_type.as_str()) {
            return Err(SignusError::UnknownCryptoError(format!("Trying to encrypt message with unknown crypto: {}", my_did.crypto_type)));
        }

        let signus = self.crypto_types.get(&my_did.crypto_type.as_str()).unwrap();

        if their_did.pk.is_none() {
            return Err(SignusError::CommonError(CommonError::InvalidStructure(format!("TheirDid doesn't contain pk: {}", their_did.did))));
        }

        let public_key = their_did.pk.clone().unwrap();

        let nonce = signus.gen_nonce();

        let secret_key = Base58::decode(&my_did.sk)?;
        let public_key = Base58::decode(&public_key)?;

        let encrypted_doc = signus.encrypt(&secret_key, &public_key, &doc.as_bytes(), &nonce);
        let encrypted_doc = Base58::encode(&encrypted_doc);
        let nonce = Base58::encode(&nonce);

        Ok((encrypted_doc, nonce))
    }

    pub fn decrypt(&self, my_did: &MyDid, their_did: &TheirDid, doc: &str, nonce: &str) -> Result<String, SignusError> {
        if !self.crypto_types.contains_key(&my_did.crypto_type.as_str()) {
            return Err(SignusError::UnknownCryptoError(format!("MyDid crypto is unknown: {}, {}", my_did.did, my_did.crypto_type)));
        }

        let signus = self.crypto_types.get(&my_did.crypto_type.as_str()).unwrap();

        if their_did.pk.is_none() {
            return Err(SignusError::CommonError(
                CommonError::InvalidStructure(format!("No pk in TheirDid: {}", their_did.did))));
        }

        let public_key = their_did.pk.clone().unwrap();

        let secret_key = Base58::decode(&my_did.sk)?;
        let public_key = Base58::decode(&public_key)?;
        let nonce = Base58::decode(&nonce)?;
        let doc = Base58::decode(&doc)?;

        let decrypted_doc = signus.decrypt(&secret_key, &public_key, &doc, &nonce)?;

        let decrypted_doc = str::from_utf8(&decrypted_doc)
            .map_err(|err|
                CommonError::InvalidStructure(format!("Decrypted message is invalid string: {}", their_did.did)))?;
        Ok(decrypted_doc.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use services::signus::types::MyDidInfo;

    #[test]
    fn create_my_did_with_empty_input_works() {
        let service = SignusService::new();
        let did_info = MyDidInfo::new(None, None, None, None);

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
    }

    #[test]
    fn create_my_did_with_did_in_input_works() {
        let service = SignusService::new();

        let did = Some("Dbf2fjCbsiq2kfns".to_string());
        let did_info = MyDidInfo::new(did.clone(), None, None, None);

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());

        assert_eq!(did.unwrap(), did_info.did.unwrap());
    }

    #[test]
    fn try_create_my_did_with_invalid_crypto_type() {
        let service = SignusService::new();

        let did = Some("Dbf2fjCbsiq2kfns".to_string());
        let crypto_type = Some("type".to_string());

        let did_info = MyDidInfo::new(did.clone(), None, crypto_type, None);

        let res = service.create_my_did(&did_info);
        assert!(res.is_err());
    }

    #[test]
    fn create_my_did_with_seed_type() {
        let service = SignusService::new();

        let did = Some("Dbf2fjCbsiq2kfns".to_string());
        let seed = Some("DJASbewkdUY3265HJFDSbds278sdDSnA".to_string());

        let did_info_with_seed = MyDidInfo::new(did.clone(), seed, None, None);
        let did_info_without_seed = MyDidInfo::new(did.clone(), None, None, None);

        let res_with_seed = service.create_my_did(&did_info_with_seed);
        let res_without_seed = service.create_my_did(&did_info_without_seed);

        assert!(res_with_seed.is_ok());
        assert!(res_without_seed.is_ok());

        assert_ne!(res_with_seed.unwrap().verkey, res_without_seed.unwrap().verkey)
    }

    #[test]
    fn sign_works() {
        let service = SignusService::new();

        let did_info = MyDidInfo::new(None, None, None, None);

        let message = r#"{
            "reqId":1495034346617224651,
            "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
            "operation":{
                "type":"1",
                "dest":"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB"
            }
        }"#;

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
        let my_did = res.unwrap();

        let signature = service.sign(&my_did, message);
        assert!(signature.is_ok());
    }

    #[test]
    fn sign_works_for_invalid_signkey() {
        let service = SignusService::new();

        let message = r#"{
            "reqId":1495034346617224651,
            "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
            "operation":{
                "type":"1",
                "dest":"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB"
            }
        }"#;

        let my_did = MyDid::new("NcYxiDXkpYi6ov5FcYDi1e".to_string(),
                                DEFAULT_CRYPTO_TYPE.to_string(),
                                "pk".to_string(),
                                "sk".to_string(),
                                "verkey".to_string(),
                                "signkey".to_string());

        let signature = service.sign(&my_did, message);
        assert!(signature.is_err());
    }

    #[test]
    fn sign_verify_works() {
        let service = SignusService::new();

        let did_info = MyDidInfo::new(None, None, None, None);

        let message = r#"{
            "reqId":1495034346617224651,
            "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
            "operation":{
                "type":"1",
                "dest":"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB"
            }
        }"#;

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
        let my_did = res.unwrap();

        let signature = service.sign(&my_did, message);
        assert!(signature.is_ok());
        let signature = signature.unwrap();

        let their_did = TheirDid {
            did: "sw2SA2jCbsiq2kfns".to_string(),
            crypto_type: DEFAULT_CRYPTO_TYPE.to_string(),
            pk: None,
            endpoint: None,
            verkey: Some(my_did.verkey)
        };

        let res = service.verify(&their_did, &signature);
        assert!(res.is_ok());
        let valid = res.unwrap();
        assert!(valid);
    }

    #[test]
    fn try_verify_with_invalid_verkey() {
        let service = SignusService::new();

        let did_info = MyDidInfo::new(None, None, None, None);

        let message = r#"{
            "reqId":1495034346617224651,
            "identifier":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
            "operation":{
                "type":"1",
                "dest":"4efZu2SXufS556yss7W5k6Po37jt4371RM4whbPKBKdB"
            }
        }"#;

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
        let my_did = res.unwrap();

        let signature = service.sign(&my_did, message);
        assert!(signature.is_ok());
        let signature = signature.unwrap();

        let their_did = TheirDid {
            did: "sw2SA2jCbsiq2kfns".to_string(),
            crypto_type: DEFAULT_CRYPTO_TYPE.to_string(),
            pk: None,
            endpoint: None,
            verkey: Some("AnnxV4t3LUHKZaxVQDWoVaG44NrGmeDYMA4Gz6C2tCZd".to_string())
        };

        let res = service.verify(&their_did, &signature);
        assert!(res.is_ok());
        assert_eq!(false, res.unwrap());
    }

    #[test]
    fn encrypt_works() {
        let service = SignusService::new();

        let msg = "some message";

        let did_info = MyDidInfo::new(None, None, None, None);

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
        let my_did = res.unwrap();


        let res = service.create_my_did(&did_info.clone());
        assert!(res.is_ok());
        let their_did = res.unwrap();

        let their_did = TheirDid {
            did: their_did.did,
            crypto_type: DEFAULT_CRYPTO_TYPE.to_string(),
            pk: Some(their_did.pk),
            endpoint: None,
            verkey: Some(their_did.verkey)
        };

        let encrypted_message = service.encrypt(&my_did, &their_did, msg);
        assert!(encrypted_message.is_ok());
    }

    #[test]
    fn encrypt_decrypt_works() {
        let service = SignusService::new();

        let msg = "some message";

        let did_info = MyDidInfo::new(None, None, None, None);

        let res = service.create_my_did(&did_info);
        assert!(res.is_ok());
        let my_did = res.unwrap();

        let my_did_for_encrypt = my_did.clone();

        let their_did_for_decrypt = TheirDid {
            did: my_did.did,
            crypto_type: DEFAULT_CRYPTO_TYPE.to_string(),
            pk: Some(my_did.pk),
            endpoint: None,
            verkey: Some(my_did.verkey)
        };


        let res = service.create_my_did(&did_info.clone());
        assert!(res.is_ok());
        let their_did = res.unwrap();

        let my_did_for_decrypt = their_did.clone();

        let their_did_for_encrypt = TheirDid {
            did: their_did.did,
            crypto_type: DEFAULT_CRYPTO_TYPE.to_string(),
            pk: Some(their_did.pk),
            endpoint: None,
            verkey: Some(their_did.verkey)
        };

        let encrypted_message = service.encrypt(&my_did_for_encrypt, &their_did_for_encrypt, msg);
        assert!(encrypted_message.is_ok());
        let (encrypted_message, noce) = encrypted_message.unwrap();

        let decrypted_message = service.decrypt(&my_did_for_decrypt, &their_did_for_decrypt, &encrypted_message, &noce);
        assert!(decrypted_message.is_ok());
        let decrypted_message = decrypted_message.unwrap();

        assert_eq!(msg.to_string(), decrypted_message);
    }
}