//
//  AnoncredsUtils.m
//  libsovrin-demo
//
//  Created by Kirill Neznamov on 24/05/2017.
//  Copyright © 2017 Kirill Neznamov. All rights reserved.
//

#import "AnoncredsUtils.h"
#import <libsovrin/libsovrin.h>
#import "TestUtils.h"
#import "WalletUtils.h"

@implementation AnoncredsUtils

+ (AnoncredsUtils *)sharedInstance
{
    static AnoncredsUtils *instance = nil;
    static dispatch_once_t dispatch_once_block;
    
    dispatch_once(&dispatch_once_block, ^{
        instance = [AnoncredsUtils new];
    });
    
    return instance;
}

- (NSString *)getGvtSchemaJson:(NSNumber *)seqNo
{
    return [NSString stringWithFormat:@"{"\
                                       "\"name\":\"gvt\"," \
                                       "\"version\":\"1.0\"," \
                                       "\"keys\":[\"age\",\"sex\",\"height\",\"name\"]," \
                                       "\"seq_no\":%ld" \
                                       "}", [seqNo integerValue]
    ];
}

- (NSString *)getClaimOfferJson:(NSString *)issuerDid
                          seqNo:(NSNumber *)claimDefSeqNo
{
    return [NSString stringWithFormat:@"{"\
            "\"issuer_did\":\"%@\"," \
            "\"claim_def_seq_no\":%ld" \
            "}", issuerDid, [claimDefSeqNo integerValue]
            ];
}

- (NSString *)getGvtClaimJson
{
    return [NSString stringWithFormat:@"{"\
                                       "\"sex\":[\"male\",\"5944657099558967239210949258394887428692050081607692519917050011144233115103\"],"\
                                       "\"name\":[\"Alex\",\"1139481716457488690172217916278103335\"],"\
                                       "\"height\":[\"175\",\"175\"],"\
                                       "\"age\":[\"28\",\"28\"]"\
                                       "}"];
}

- (NSString *)getXyzSchemaJson:(NSNumber *)schemaSeqNo
{
    return [NSString stringWithFormat:@"{"\
            "\"name\":\"xyz\","\
            "\"version\":\"1.0\","\
            "\"keys\":[\"status\",\"period\"],"\
            "\"seq_no\":%ld"\
            "}",[schemaSeqNo integerValue]];
}

- (NSString *)getXyzClaimJson
{
    return [NSString stringWithFormat:@"{"\
                                       "  \"status\":[\"partial\",\"51792877103171595686471452153480627530895\"],"\
                                       "  \"period\":[\"8\",\"8\"]"\
                                       "}"];
}

// MARK: issuer claim
- (NSError *)issuerCreateClaimWithWalletHandle:(SovrinHandle)walletHandle
                                     claimJson:(NSString *)claimJson
                                  claimReqJson:(NSString *)claimReqJson
                                  outClaimJson:(NSString **)xClaimJson
                         outRevocRegUpdateJSON:(NSString **)revocRegUpdateJSON
{
    __block NSError *err = nil;
    __block NSString *outClaimJson;
    __block NSString *outRevocRegUpdateJSON;
    XCTestExpectation* completionExpectation = nil;

    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];

    NSError *ret = [SovrinAnoncreds  issuerCreateClaimWithWalletHandle:walletHandle
                                                          claimReqJSON:claimReqJson
                                                             claimJSON:claimJson
                                                         revocRegSeqNo:@(-1)
                                                        userRevocIndex:@(-1)
                                                            completion:^(NSError *error, NSString *revocRegUpdateJSON, NSString *claimJSON)
    {
        err = error;
        outRevocRegUpdateJSON = revocRegUpdateJSON;
        outClaimJson = claimJSON;
        [completionExpectation fulfill];
    }];
    
    
    if( ret.code != Success)
    {
        return ret;
    }

    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils shortTimeout]];
    
    *xClaimJson = outClaimJson;
    *revocRegUpdateJSON = outRevocRegUpdateJSON;
    return err;
}

- (NSError *)issuerCreateClaimDefinifionWithWalletHandle:(SovrinHandle)walletHandle
                                              schemaJson:(NSString *)schemaJson
                                            claimDefJson:(NSString **)claimDefJson
                                            claimDefUUID:(NSString **)claimDefUUID;
{
    __block NSError *err = nil;
    __block NSString *outClaimDefJson = nil;
    __block NSString *outClaimDefUUID = nil;
    XCTestExpectation *completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [SovrinAnoncreds  issuerCreateAndStoreClaimDefWithWalletHandle:walletHandle
                                                                       schemaJSON:schemaJson
                                                                    signatureType:nil
                                                                   createNonRevoc:NO
                                                                       completion:^(NSError *error, NSString *claimDefJSON, NSString *claimDefUUID)
                    {
                        err = error;
                        outClaimDefJson = claimDefJSON;
                        outClaimDefUUID = claimDefUUID;
                        
                        [completionExpectation fulfill];
                    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    
    if ( claimDefJson ) { *claimDefJson = outClaimDefJson; }
    if ( claimDefUUID ) { *claimDefUUID = outClaimDefUUID; }
    
    return err;
}


- (NSError *) createClaimDefinitionAndSetLink:(SovrinHandle)walletHandle
                                       schema:(NSString *)schema
                                        seqNo:(NSNumber *)claimDefSeqNo
                                      outJson:(NSString **)outJson
{
    NSString *json = nil;
    NSString *uuid;
    NSError *ret;
    
    ret = [self issuerCreateClaimDefinifionWithWalletHandle:walletHandle
                                                 schemaJson:schema
                                               claimDefJson:&json
                                               claimDefUUID:&uuid];
    if( ret.code != Success )
    {
        return ret;
    }
    
    ret = [[WalletUtils sharedInstance] walletSetSeqNoForValue:walletHandle
                                            claimDefUUID:uuid
                                           claimDefSeqNo:claimDefSeqNo];
    *outJson = json;
    return ret;
}

- (NSArray *)getUniqueClaimsFrom:(NSDictionary *)proofClaims
{
    NSMutableArray* uniqueClaims =  [[NSMutableArray alloc] init];
    
    for (NSDictionary* claims in proofClaims.allValues )
    {
    
        for (NSArray* claim in claims.allValues)
        {
            if ( ![uniqueClaims containsObject: claim[0]] )
            {
                [uniqueClaims addObject:claim[0]];
            }
        }
    }
    
    NSArray* res = [NSArray arrayWithArray:uniqueClaims];
    return res;
    
}

- (NSError *)proverCreateMasterSecret:(SovrinHandle)walletHandle
                     masterSecretName:(NSString *)name
{
    __block NSError *err = nil;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [SovrinAnoncreds proverCreateMasterSecretWithWalletHandle:walletHandle
                                                            masterSecretName:name
                                                                  completion:^(NSError *error)
    {
        err = error;
        [completionExpectation fulfill];
    }];

    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    return err;
}

- (NSError *)proverStoreClaimOffer:(SovrinHandle)walletHandle
                    claimOfferJson:(NSString *)str
{
    __block NSError *err = nil;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [SovrinAnoncreds proverStoreClaimOfferWithWalletHandle:walletHandle
                                                           claimOfferJSON:str
                                                               completion: ^(NSError *error)
    {
        err = error;
        [completionExpectation fulfill];
    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];

    return err;
 
}

- (NSError *)proverGetClaimOffers:(SovrinHandle)walletHandle
                       filterJson:(NSString *)filterJson
               outClaimOffersJSON:(NSString **)outJson
{
    __block NSString *json;
    __block NSError *err = nil;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [ SovrinAnoncreds proverGetClaimOffersWithWalletHandle:walletHandle
                                                               filterJSON:filterJson
                                                               completion:^(NSError *error, NSString *claimOffersJSON)
    {
        err = error;
        json = claimOffersJSON;
        [completionExpectation fulfill];
    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    
    *outJson = json;
    return err;
}

- (NSError *)proverCreateAndStoreClaimReq:(SovrinHandle)walletHandle
                                proverDid:(NSString *)pd
                           claimOfferJson:(NSString *)coj
                             claimDefJson:(NSString *)cdj
                         masterSecretName:(NSString *)name
                          outClaimReqJson:(NSString **)outJson
{
    __block NSError *err = nil;
    __block NSString *json;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [ SovrinAnoncreds proverCreateAndStoreClaimReqWithWalletHandle: walletHandle
                                                                        proverDid:pd
                                                                   claimOfferJSON:coj
                    
                                                                     claimDefJSON:cdj
                                                                 masterSecretName:name
                                                                       completion:^(NSError* error, NSString* claimReqJSON)
    {
        err = error;
        json = claimReqJSON;
        [completionExpectation fulfill];
    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    
    *outJson = json;
    return err;
}



- (NSError *) proverStoreClaimWithWalletHandle:(SovrinHandle)walletHandle
                                    claimsJson:(NSString *)str
{
    __block NSError *err = nil;
    XCTestExpectation* completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [SovrinAnoncreds proverStoreClaimWithWalletHandle:walletHandle
                                                          claimsJSON:str
                                                          completion:^(NSError *error)
    {
        XCTAssertEqual(err.code, Success, @"proverStoreClaim failed!");
        err = error;
        [completionExpectation fulfill];
    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    
    return err;
}

- (NSError *)proverGetClaimsForProofReqWithWalletHandle:(SovrinHandle)walletHandle
                                       proofRequestJson:(NSString *)str
                                          outClaimsJson:(NSString **)outClaimsJson
{
    __block NSError *err = nil;
    __block NSString *outJson;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];

    NSError *ret = [SovrinAnoncreds proverGetClaimsForProofReqWithWalletHandle:walletHandle
                                                                  proofReqJSON:str
                                                                    completion:^(NSError *error, NSString *claimsJSON)
    {
        err = error;
        outJson = claimsJSON;
        [completionExpectation fulfill];
    }];
    
    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    
    *outClaimsJson = outJson;
    
    return err;
}

- (NSError *)proverCreateProofWithWalletHandle:(SovrinHandle)walletHandle
                                  proofReqJson:(NSString *)proofReqJson
                           requestedClaimsJson:(NSString *)requestedClaimsJson
                                   schemasJson:(NSString *)schemasJson
                              masterSecretName:(NSString *)masterSecreteName
                                 claimDefsJson:(NSString *)claimDefsJson
                                 revocRegsJson:(NSString *)revocRegsJson
                                  outProofJson:(NSString **)outProofJson
{
    __block NSError *err = nil;
    XCTestExpectation* completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
   
    NSError *ret = [SovrinAnoncreds proverCreateProofWithWalletHandle:walletHandle
                                                         proofReqJSON:proofReqJson
                                                  requestedClaimsJSON:requestedClaimsJson
                                                          schemasJSON:schemasJson
                                                     masterSecretName:masterSecreteName
                                                        claimDefsJSON:claimDefsJson
                                                        revocRegsJSON:revocRegsJson
                                                           completion:^(NSError *error, NSString *proofJSON)
    {
        err = error;
        if (outProofJson)
        {
            *outProofJson = proofJSON;
        }
        [completionExpectation fulfill];
    }];

    if( ret.code != Success)
    {
        return ret;
    }

    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    return err;
}

- (NSError *)verifierVerifyProof:(NSString *)proofRequestJson
                       proofJson:(NSString *)proofJson
                     schemasJson:(NSString *)schemasJson
                   claimDefsJson:(NSString *)claimDefsJson
                   revocRegsJson:(NSString *)revocRegsJson
                        outValid:(BOOL *)isValid
{
    __block NSError *err = nil;
    XCTestExpectation *completionExpectation = nil;
    
    completionExpectation = [[ XCTestExpectation alloc] initWithDescription: @"completion finished"];
    
    NSError *ret = [SovrinAnoncreds verifierVerifyProofWithWalletHandle:proofRequestJson
                                                              proofJSON:proofJson
                                                            schemasJSON:schemasJson
                                                          claimDefsJSON:claimDefsJson
                                                          revocRegsJSON:revocRegsJson
                                                             completion:^(NSError *error, BOOL valid)
    {
        err = error;
        if(isValid)
        {
            *isValid = valid;
        }
        [completionExpectation fulfill];
    }];

    if( ret.code != Success)
    {
        return ret;
    }
    
    [self waitForExpectations: @[completionExpectation] timeout:[TestUtils defaultTimeout]];
    return err;
    
}

@end
