#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Client {
    id: u64,
    name: String,
    email: String,
    phone: String,
    order_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

struct Supplier {
    id: u64,
    name: String,
    email: String,
    phone: String,
    order_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

struct Order {
    id: u64,
    title: String,
    client_id: u64,
    supplier_id: u64,
    products: HashMap<String, u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

impl Storable for Client {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// another trait that must be implemented for a struct that is stored in a stable struct
impl BoundedStorable for Client {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}
