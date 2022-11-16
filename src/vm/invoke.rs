use std::rc::Rc;

use soroban_env_host::{
    budget::Budget,
    storage::{SnapshotSource, Storage},
    xdr::{
        self, AccountId, ContractDataEntry, Hash, HostFunction, LedgerEntry, LedgerEntryData,
        LedgerEntryExt, LedgerKey, LedgerKeyContractData, PublicKey, ScContractCode,
        ScHostStorageErrorCode, ScObject, ScStatic, ScStatus, ScVal, Uint256,
    },
    Host, HostError, Status,
};

pub fn invoke(code: Vec<u8>, id: String, function: String) -> String {
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
    let hex_id = hex::decode(&id).unwrap();
    let storage = Storage::with_recording_footprint(Rc::new(CodeOnlySnapshotSource(
        (&hex_id).try_into().unwrap(),
        ScContractCode::Wasm(code.try_into().unwrap()),
    )));
    let h = Host::with_storage_and_budget(storage, Budget::default());
    h.set_source_account(AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]))));
    let result = h.invoke_function(
        HostFunction::InvokeContract,
        vec![
            ScVal::Object(Some(ScObject::Bytes(hex_id.try_into().unwrap()))),
            ScVal::Symbol((&function).try_into().unwrap()),
        ]
        .try_into()
        .unwrap(),
    );
    match result {
        Ok(result) => serde_json::to_string_pretty(&result).unwrap(),
        Err(err) => err.to_string(),
    }

    // let (storage, budget, events) = h.try_finish().map_err(|_h| {
    //     HostError::from(ScStatus::HostStorageError(
    //         ScHostStorageErrorCode::UnknownError,
    //     ))
    // })?;

    //     eprintln!(
    //         "Footprint: {}",
    //         serde_json::to_string(&create_ledger_footprint(&storage.footprint)).unwrap(),
    //     );

    // if self.cost {
    //     eprintln!("Cpu Insns: {}", budget.get_cpu_insns_count());
    //     eprintln!("Mem Bytes: {}", budget.get_mem_bytes_count());
    //     for cost_type in CostType::variants() {
    //         eprintln!("Cost ({cost_type:?}): {}", budget.get_input(*cost_type));
    //     }
    // }

    // for (i, event) in events.0.iter().enumerate() {
    //     eprint!("#{i}: ");
    //     match event {
    //         HostEvent::Contract(e) => {
    //             eprintln!("event: {}", serde_json::to_string(&e).unwrap());
    //         }
    //         HostEvent::Debug(e) => eprintln!("debug: {e}"),
    //     }
    // }
}
