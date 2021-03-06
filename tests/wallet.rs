extern crate sovrin;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

#[macro_use]
mod utils;

use utils::wallet::WalletUtils;
use utils::signus::SignusUtils;
use utils::test::TestUtils;

use sovrin::api::ErrorCode;

mod high_cases {
    use super::*;

    mod create_wallet {
        use super::*;

        #[test]
        fn sovrin_create_wallet_works() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works";
            let wallet_name = "wallet1";
            let xtype = "default";

            WalletUtils::create_wallet(pool_name, wallet_name, Some(xtype), None).unwrap();

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_create_wallet_works_for_unknown_type() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works_for_unknown_type";
            let wallet_name = "wallet1";
            let xtype = "type";

            let res = WalletUtils::create_wallet(pool_name, wallet_name, Some(xtype), None);
            assert_eq!(res.unwrap_err(), ErrorCode::WalletUnknownTypeError);

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_create_wallet_works_for_empty_type() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works_for_empty_type";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_create_wallet_works_for_config() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works";
            let wallet_name = "wallet1";
            let xtype = "default";
            let config = r#"{"freshness_time":1000}"#;

            WalletUtils::create_wallet(pool_name, wallet_name, Some(xtype), Some(config)).unwrap();

            TestUtils::cleanup_storage();
        }
    }

    mod delete_wallet {
        use super::*;

        #[test]
        fn sovrin_delete_wallet_works() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_delete_wallet_works";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            WalletUtils::delete_wallet(wallet_name).unwrap();
            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();

            TestUtils::cleanup_storage();
        }
    }

    mod open_wallet {
        use super::*;

        #[test]
        fn sovrin_open_wallet_works() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_open_wallet_works";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            WalletUtils::open_wallet(wallet_name, None).unwrap();

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_open_wallet_works_for_config() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_open_wallet_works";
            let wallet_name = "wallet1";
            let config = r#"{"freshness_time":1000}"#;

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            WalletUtils::open_wallet(wallet_name, Some(config)).unwrap();

            TestUtils::cleanup_storage();
        }
    }

    mod close_wallet {
        use super::*;

        #[test]
        fn sovrin_close_wallet_works() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_close_wallet_works";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();

            let wallet_handle = WalletUtils::open_wallet(wallet_name, None).unwrap();
            WalletUtils::close_wallet(wallet_handle).unwrap();
            WalletUtils::open_wallet(wallet_name, None).unwrap();

            TestUtils::cleanup_storage();
        }
    }

    mod set_seqno_wallet {
        use super::*;

        #[test]
        fn sovrin_wallet_set_seqno_works() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_wallet_set_seqno_works";
            let wallet_name = "wallet1";
            let xtype = "default";

            let wallet_handle = WalletUtils::create_and_open_wallet(pool_name, wallet_name, xtype).unwrap();

            let (did, _, _) = SignusUtils::create_my_did(wallet_handle, "{}").unwrap();

            WalletUtils::wallet_set_seq_no_for_value(wallet_handle, &did, 1).unwrap();

            TestUtils::cleanup_storage();
        }
    }
}

mod medium_cases {
    use super::*;

    mod create_wallet {
        use super::*;

        #[test]
        fn sovrin_create_wallet_works_for_duplicate_name() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works_for_duplicate_name";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            let res = WalletUtils::create_wallet(pool_name, wallet_name, None, None);
            assert_eq!(res.unwrap_err(), ErrorCode::WalletAlreadyExistsError);

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_create_wallet_works_for_empty_name() {
            TestUtils::cleanup_storage();

            let pool_name = "pool";
            let wallet_name = "";

            let res = WalletUtils::create_wallet(pool_name, wallet_name, None, None);
            assert_eq!(res.unwrap_err(), ErrorCode::CommonInvalidParam3);

            TestUtils::cleanup_storage();
        }
    }

    mod delete_wallet {
        use super::*;

        #[test]
        fn sovrin_delete_wallet_works_for_invalid_wallet_name() {
            TestUtils::cleanup_storage();

            let res = WalletUtils::delete_wallet("wallet1");
            assert_eq!(res.unwrap_err(), ErrorCode::CommonIOError);

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_delete_wallet_works_for_twice() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_delete_wallet_works_for_deleted_wallet";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            WalletUtils::delete_wallet(wallet_name).unwrap();
            let res = WalletUtils::delete_wallet(wallet_name);
            assert_eq!(res.unwrap_err(), ErrorCode::CommonIOError);

            TestUtils::cleanup_storage();
        }
    }

    mod open_wallet {
        use super::*;

        #[test]
        fn sovrin_open_wallet_works_for_not_created_wallet() {
            TestUtils::cleanup_storage();

            let res = WalletUtils::open_wallet("wallet1", None);
            assert_eq!(res.unwrap_err(), ErrorCode::CommonIOError);

            TestUtils::cleanup_storage();
        }

        #[test]
        #[ignore] //TODO Check is not implemented
        fn sovrin_open_wallet_works_for_twice() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works";
            let wallet_name = "wallet1";

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();

            WalletUtils::open_wallet(wallet_name, None).unwrap();
            let res =  WalletUtils::open_wallet(wallet_name, None);
            assert_eq!(res.unwrap_err(), ErrorCode::CommonIOError);

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_open_wallet_works_for_two_wallets() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_create_wallet_works";
            let wallet_name_1 = "wallet1";
            let wallet_name_2 = "wallet2";

            WalletUtils::create_wallet(pool_name, wallet_name_1, None, None).unwrap();
            WalletUtils::create_wallet(pool_name, wallet_name_2, None, None).unwrap();
            WalletUtils::open_wallet(wallet_name_1, None).unwrap();
            WalletUtils::open_wallet(wallet_name_2, None).unwrap();

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_open_wallet_works_for_invalid_config() {
            TestUtils::cleanup_storage();

            let pool_name = "sovrin_open_wallet_works";
            let wallet_name = "wallet1";
            let config = r#"{"field":"value"}"#;

            WalletUtils::create_wallet(pool_name, wallet_name, None, None).unwrap();
            let res = WalletUtils::open_wallet(wallet_name, Some(config));
            assert_eq!(res.unwrap_err(), ErrorCode::CommonInvalidStructure);

            TestUtils::cleanup_storage();
        }
    }

    mod close_wallet {
        use super::*;

        #[test]
        fn sovrin_close_wallet_works_for_invalid_handle() {
            TestUtils::cleanup_storage();

            let res = WalletUtils::close_wallet(1);
            assert_eq!(res.unwrap_err(), ErrorCode::WalletInvalidHandle);

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_close_wallet_works_for_twice() {
            TestUtils::cleanup_storage();

            let wallet_handle = WalletUtils::create_and_open_wallet("sovrin_close_wallet_works_for_closed_wallet", "wallet1", "default").unwrap();

            WalletUtils::close_wallet(wallet_handle).unwrap();
            let res = WalletUtils::close_wallet(wallet_handle);
            assert_eq!(res.unwrap_err(), ErrorCode::WalletInvalidHandle);

            TestUtils::cleanup_storage();
        }
    }

    mod set_seqno {
        use super::*;

        #[test]
        fn sovrin_wallet_set_seqno_works_for_not_exists_key() {
            TestUtils::cleanup_storage();

            let wallet_handle = WalletUtils::create_and_open_wallet("sovrin_wallet_set_seqno_works_for_not_exists_key", "wallet1", "default").unwrap();

            //TODO may be we must return WalletNotFound in case if key not exists in wallet
            WalletUtils::wallet_set_seq_no_for_value(wallet_handle, "key", 1).unwrap();

            TestUtils::cleanup_storage();
        }

        #[test]
        fn sovrin_wallet_set_seqno_works_for_invalid_wallet() {
            TestUtils::cleanup_storage();

            let wallet_handle = WalletUtils::create_and_open_wallet("sovrin_wallet_set_seqno_works_for_invalid_wallet", "wallet1", "default").unwrap();


            let invalid_wallet_handle = wallet_handle + 1;
            let res = WalletUtils::wallet_set_seq_no_for_value(invalid_wallet_handle, "key", 1);
            assert_eq!(res.unwrap_err(), ErrorCode::WalletInvalidHandle);

            TestUtils::cleanup_storage();
        }
    }
}
