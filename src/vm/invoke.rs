use std::rc::Rc;

use soroban_env_host::{
    budget::Budget,
    storage::{SnapshotSource, Storage},
    xdr::{
        self, AccountId, ContractDataEntry, Hash, HostFunction, LedgerEntry, LedgerEntryData,
        LedgerEntryExt, LedgerKey, LedgerKeyContractData, PublicKey, ScContractCode,
        ScHostStorageErrorCode, ScObject, ScStatic, ScStatus, ScVal, Uint256,
    },
    Host, HostError, MeteredOrdMap, Status, events::Events,
};

pub fn invoke(
    source: Option<MeteredOrdMap<Box<LedgerKey>, Option<Box<LedgerEntry>>>>,
    code: Vec<u8>,
    id: String,
    function: String,
    args: Vec<ScVal>,
) -> (
    String,
    MeteredOrdMap<Box<LedgerKey>, Option<Box<LedgerEntry>>>,
    Budget,
    Events,
) {
    let hex_id = hex::decode(&id).unwrap();
    let mut sources: Vec<Box<dyn SnapshotSource>> = vec![Box::new(CodeOnlySnapshotSource(
        (&hex_id).try_into().unwrap(),
        ScContractCode::Wasm(code.try_into().unwrap()),
    ))];
    if let Some(incoming_source) = source {
        sources.push(Box::new(MemorySnapshotSource(incoming_source)));
    }
    let source = MultiSnapshotSource(sources);
    let storage = Storage::with_recording_footprint(Rc::new(source));
    let h = Host::with_storage_and_budget(storage, Budget::default());
    h.set_source_account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]))));
    let result = h.invoke_function(
        HostFunction::InvokeContract,
        [
            vec![
                ScVal::Object(Some(ScObject::Bytes(hex_id.try_into().unwrap()))),
                ScVal::Symbol((&function).try_into().unwrap()),
            ],
            args,
        ]
        .concat()
        .try_into()
        .unwrap(),
    );
    let result_str = match result {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap(),
        Err(err) => err.to_string(),
    };

    let (Storage { map: storage, .. }, budget, events) = h
        .try_finish()
        .map_err(|_h| {
            HostError::from(ScStatus::HostStorageError(
                ScHostStorageErrorCode::UnknownError,
            ))
        })
        .unwrap();

    (result_str, storage, budget, events)
}

struct MultiSnapshotSource(Vec<Box<dyn SnapshotSource>>);
impl SnapshotSource for MultiSnapshotSource {
    fn get(&self, key: &xdr::LedgerKey) -> Result<xdr::LedgerEntry, HostError> {
        for s in &self.0 {
            if let Ok(le) = s.get(key) {
                return Ok(le);
            }
        }
        let status: Status =
            ScStatus::HostStorageError(ScHostStorageErrorCode::UnknownError).into();
        Err(status.into())
    }
    fn has(&self, key: &xdr::LedgerKey) -> Result<bool, HostError> {
        for s in &self.0 {
            if let Ok(true) = s.has(key) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

struct CodeOnlySnapshotSource(Hash, ScContractCode);
impl CodeOnlySnapshotSource {
    fn key(&self) -> LedgerKey {
        LedgerKey::ContractData(LedgerKeyContractData {
            contract_id: self.0.clone(),
            key: ScVal::Static(ScStatic::LedgerKeyContractCode),
        })
    }
    fn data(&self) -> LedgerEntryData {
        LedgerEntryData::ContractData(ContractDataEntry {
            contract_id: self.0.clone(),
            key: ScVal::Static(ScStatic::LedgerKeyContractCode),
            val: ScVal::Object(Some(ScObject::ContractCode(self.1.clone()))),
        })
    }
}
impl SnapshotSource for CodeOnlySnapshotSource {
    fn get(&self, key: &xdr::LedgerKey) -> Result<xdr::LedgerEntry, HostError> {
        if key == &self.key() {
            Ok(LedgerEntry {
                last_modified_ledger_seq: 0,
                data: self.data(),
                ext: LedgerEntryExt::V0,
            })
        } else {
            let status: Status =
                ScStatus::HostStorageError(ScHostStorageErrorCode::UnknownError).into();
            Err(status.into())
        }
    }
    fn has(&self, key: &xdr::LedgerKey) -> Result<bool, HostError> {
        Ok(key == &self.key())
    }
}

struct MemorySnapshotSource(MeteredOrdMap<Box<LedgerKey>, Option<Box<LedgerEntry>>>);
impl SnapshotSource for MemorySnapshotSource {
    fn get(&self, key: &xdr::LedgerKey) -> Result<xdr::LedgerEntry, HostError> {
        if let Ok(Some(Some(le))) = self.0.get(key) {
            return Ok(*le.clone());
        }
        let status: Status =
            ScStatus::HostStorageError(ScHostStorageErrorCode::UnknownError).into();
        Err(status.into())
    }
    fn has(&self, key: &xdr::LedgerKey) -> Result<bool, HostError> {
        if let Ok(Some(Some(_))) = self.0.get(key) {
            return Ok(true);
        }
        Ok(false)
    }
}
