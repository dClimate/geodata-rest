use crate::errors::Error;
// TODO: import this from geodata-anchor
use crate::common::msg::{CreateMsg, ExecuteMsg, InstantiateMsg, ValidateMsg};
use bip32::XPrv;
use bip39::{Language, Mnemonic, Seed};
use cosmrs::{
  cosmwasm::{AccessConfig, MsgExecuteContract, MsgInstantiateContract, MsgStoreCode},
  crypto::secp256k1::{self, SigningKey},
  crypto::PublicKey,
  tx::{self, AccountNumber, Fee, Msg, Raw, SignDoc, SignerInfo, Tx},
  AccountId, Coin,
};
use cosmwasm_std::Timestamp;
use lazy_static::lazy_static;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::panic;
use std::str::{self, FromStr};
use tendermint_rpc as rpc;
use tracing::{error, info};

lazy_static! {
  static ref SENDER_ACCOUNT_ID: AccountId = AccountId::from_str(&SENDER_ADDRESS).unwrap();
  static ref PK_BYTES: Vec<u8> = private_key_bytes(SENDER_PHRASE, "");
  static ref AMOUNT: Coin = Coin {
    amount: 1u8.into(),
    denom: DENOM.parse().unwrap(),
  };
  static ref FEE: Fee = Fee::from_amount_and_gas(AMOUNT.clone(), GAS);
  static ref SENDER_PUBLIC_KEY: PublicKey = secp256k1::SigningKey::from_bytes(PK_BYTES.as_slice())
    .unwrap()
    .public_key();
}
const SENDER_ADDRESS: &str = "juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y";
const SENDER_PHRASE: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";
const DENOM: &str = "ujunox";
const RPC_PORT2: u16 = 26657;
const CHAIN_ID: &str = "testing";
const MEMO: &str = "test memo";
const TIMEOUT_HEIGHT: u16 = 9001u16;
const ACCOUNT_NUMBER: AccountNumber = 1;
const GAS: u64 = 20_000_000;

/// anchors new geodata on the blockchain
pub async fn anchor_geodata(
  geodata_id: &str,
  account_id: &str,
  hash: &str,
  created_nanos: u64,
) -> Result<(), Error> {
  // TODO: move store and instantiate to tests/common
  // store
  let mut contract_code = File::open("./assets/geodata_anchor.wasm").unwrap();
  let mut buffer: Vec<u8> = Vec::new();
  contract_code.read_to_end(&mut buffer).unwrap();

  let msg_store = MsgStoreCode {
    sender: SENDER_ACCOUNT_ID.clone(),
    wasm_byte_code: buffer,
    instantiate_permission: None::<AccessConfig>,
  }
  .to_any()
  .unwrap();

  let mut sequence_number = 0;

  let tx_body = tx::Body::new(vec![msg_store], MEMO, TIMEOUT_HEIGHT);
  let auth_info = SignerInfo::single_direct(Some(SENDER_PUBLIC_KEY.clone()), sequence_number)
    .auth_info(FEE.clone());
  let sign_doc = SignDoc::new(&tx_body, &auth_info, &CHAIN_ID.parse().unwrap(), ACCOUNT_NUMBER).unwrap();

  // SigningKey cannot be maintained as a variable (doesnt implement Send, not thread-safe), so we need to recreate it each time
  let tx_raw = sign_tx(
    sign_doc,
    secp256k1::SigningKey::from_bytes(PK_BYTES.as_slice()).unwrap(),
  );

  let rpc_address = format!("http://localhost:{}", RPC_PORT2);
  let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).unwrap();

  let tx_commit_response: rpc::endpoint::broadcast::tx_commit::Response =
    tx_raw.broadcast_commit(&rpc_client).await.unwrap();

  if tx_commit_response.check_tx.code.is_err() {
    error!("check_tx failed: {:?}", tx_commit_response.check_tx);
  }

  if tx_commit_response.deliver_tx.code.is_err() {
    error!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
  }

  poll_for_tx(&rpc_client, tx_commit_response.hash).await;

  // instantiate
  let instantiate_msg = InstantiateMsg {
    admins: vec![SENDER_ADDRESS.to_string()],
    users: vec![SENDER_ADDRESS.to_string()],
    mutable: true,
  };

  let instantiate_msg_json = serde_json::to_string(&instantiate_msg).unwrap();
  let msg_instantiate = MsgInstantiateContract {
    sender: SENDER_ACCOUNT_ID.clone(),
    admin: None::<AccountId>,
    code_id: 1,
    label: Some(MEMO.to_string()),
    msg: instantiate_msg_json.as_bytes().to_vec(),
    funds: vec![AMOUNT.clone()],
  }
  .to_any()
  .unwrap();

  let tx_body = tx::Body::new(vec![msg_instantiate], MEMO, TIMEOUT_HEIGHT);
  sequence_number = sequence_number + 1;
  let auth_info = SignerInfo::single_direct(Some(SENDER_PUBLIC_KEY.clone()), sequence_number)
    .auth_info(FEE.clone());
  let sign_doc = SignDoc::new(&tx_body, &auth_info, &CHAIN_ID.parse().unwrap(), ACCOUNT_NUMBER).unwrap();
  let tx_raw = sign_tx(
    sign_doc,
    secp256k1::SigningKey::from_bytes(PK_BYTES.as_slice()).unwrap(),
  );

  let tx_commit_response: rpc::endpoint::broadcast::tx_commit::Response =
    tx_raw.broadcast_commit(&rpc_client).await.unwrap();

  if tx_commit_response.check_tx.code.is_err() {
    error!("check_tx failed: {:?}", tx_commit_response.check_tx);
  }

  if tx_commit_response.deliver_tx.code.is_err() {
    error!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
  }

  let mut contract_address: String = "invalid".to_string();
  for event in tx_commit_response.deliver_tx.events {
    if event.type_str == "instantiate" {
      contract_address = event.attributes[0].value.to_string();
      break;
    }
  }

  info!("instantiate: contract address: {:?}", contract_address);
  env::set_var("CONTRACT_ADDRESS", contract_address.clone());
  poll_for_tx(&rpc_client, tx_commit_response.hash).await;

  let created = Timestamp::from_nanos(created_nanos);

  let create_msg = CreateMsg {
    id: geodata_id.to_string(),
    account: account_id.to_string(),
    hash: hash.to_string(),
    created,
  };

  let create_execute_msg = ExecuteMsg::Create(create_msg);
  let create_execute_msg_json = serde_json::to_string(&create_execute_msg).unwrap();
  let contract_account_id = AccountId::from_str(&contract_address).unwrap();
  let msg_execute = MsgExecuteContract {
    sender: SENDER_ACCOUNT_ID.clone(),
    contract: contract_account_id.clone(),
    msg: create_execute_msg_json.as_bytes().to_vec(),
    funds: vec![AMOUNT.clone()],
  }
  .to_any()
  .unwrap();

  sequence_number = sequence_number + 1;
  let tx_body = tx::Body::new(vec![msg_execute], MEMO.to_string(), TIMEOUT_HEIGHT);
  let auth_info = SignerInfo::single_direct(Some(SENDER_PUBLIC_KEY.clone()), sequence_number)
    .auth_info(FEE.clone());
  let sign_doc = SignDoc::new(&tx_body, &auth_info, &CHAIN_ID.parse().unwrap(), ACCOUNT_NUMBER).unwrap();
  let tx_raw = sign_tx(
    sign_doc,
    secp256k1::SigningKey::from_bytes(PK_BYTES.as_slice()).unwrap(),
  );

  let tx_commit_response: rpc::endpoint::broadcast::tx_commit::Response =
    tx_raw.broadcast_commit(&rpc_client).await.unwrap();

  if tx_commit_response.check_tx.code.is_err() {
    error!("check_tx failed: {:?}", tx_commit_response.check_tx);
  }

  if tx_commit_response.deliver_tx.code.is_err() {
    error!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
  }

  poll_for_tx(&rpc_client, tx_commit_response.hash).await;
  env::set_var("CURRENT_SEQUENCE", sequence_number.to_string());
  Ok(())
}
/// validates geodata on the blockchain
pub async fn validate_geodata(
  geodata_id: &str,
  account_id: &str,
  hash: &str,
  created_nanos: u64,
) -> Result<(), Error> {
  let contract_address = env::var("CONTRACT_ADDRESS").unwrap();
  let contract_account_id = AccountId::from_str(&contract_address).unwrap();
  let sequence_number: u64 = env::var("CURRENT_SEQUENCE").unwrap().parse::<u64>().unwrap() + 1;

  let created = Timestamp::from_nanos(created_nanos);
  let validate_msg = ValidateMsg {
    id: geodata_id.to_string(),
    account: account_id.to_string(),
    hash: hash.to_string(),
    created,
  };
  let validate_execute_msg = ExecuteMsg::Validate(validate_msg);
  let validate_execute_msg_json = serde_json::to_string(&validate_execute_msg).unwrap();
  let msg_execute = MsgExecuteContract {
    sender: SENDER_ACCOUNT_ID.clone(),
    contract: contract_account_id.clone(),
    msg: validate_execute_msg_json.as_bytes().to_vec(),
    funds: vec![AMOUNT.clone()],
  }
  .to_any()
  .unwrap();
  let tx_body = tx::Body::new(vec![msg_execute], MEMO.to_string(), TIMEOUT_HEIGHT);
  let auth_info =
    SignerInfo::single_direct(Some(SENDER_PUBLIC_KEY.clone()), sequence_number).auth_info(FEE.clone());
  let sign_doc = SignDoc::new(&tx_body, &auth_info, &CHAIN_ID.parse().unwrap(), ACCOUNT_NUMBER).unwrap();
  let tx_raw = sign_tx(
    sign_doc,
    secp256k1::SigningKey::from_bytes(PK_BYTES.as_slice()).unwrap(),
  );

  let rpc_address = format!("http://localhost:{}", RPC_PORT2);
  let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).unwrap();

  let tx_commit_response: rpc::endpoint::broadcast::tx_commit::Response =
    tx_raw.broadcast_commit(&rpc_client).await.unwrap();

  if tx_commit_response.check_tx.code.is_err() {
    error!("check_tx failed: {:?}", tx_commit_response.check_tx);
  }

  if tx_commit_response.deliver_tx.code.is_err() {
    error!("deliver_tx failed: {:?}", tx_commit_response.deliver_tx);
  }

  poll_for_tx(&rpc_client, tx_commit_response.hash).await;
  Ok(())
}

fn private_key_bytes(mnemonic: &str, passphrase: &str) -> Vec<u8> {
  let seed = Seed::new(
    &Mnemonic::from_phrase(mnemonic, Language::English).unwrap(),
    passphrase,
  );
  let privk = XPrv::derive_from_path(&seed, &"m/44'/118'/0'/0/0".parse().unwrap()).unwrap();
  let bytes = privk.private_key().to_bytes();
  bytes.to_vec()
}

fn sign_tx(sign_doc: SignDoc, signing_key: SigningKey) -> Raw {
  sign_doc.sign(&signing_key).unwrap()
}

/// Wait for a transaction with the given hash to appear in the blockchain
async fn poll_for_tx(rpc_client: &rpc::HttpClient, tx_hash: tx::Hash) -> Tx {
  let attempts = 20;

  for _ in 0..attempts {
    if let Ok(tx) = Tx::find_by_hash(rpc_client, tx_hash).await {
      return tx;
    }
  }

  panic!("couldn't find transaction after {} attempts!", attempts);
}
