#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
// use std::fmt::format;
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

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Supplier {
    id: u64,
    name: String,
    email: String,
    phone: String,
    order_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Order {
    id: u64,
    title: String,
    client_id: u64,
    supplier_id: Option<u64>,
    products: HashMap<String, u64>,
    is_complete: bool,
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

impl Storable for Supplier {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl Storable for Order {
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

impl BoundedStorable for Supplier {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl BoundedStorable for Order {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static CLIENT_STORAGE: RefCell<StableBTreeMap<u64, Client, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static SUPPLIER_STORAGE: RefCell<StableBTreeMap<u64, Supplier, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static ORDERS: RefCell<StableBTreeMap<u64, Order, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ClientPayload {
    name: String,
    email: String,
    phone: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct SupplierPayload {
    name: String,
    email: String,
    phone: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct OrderPayload {
    title: String,
    client_id: u64,
    supplier_id: u64,
    products: HashMap<String, u64>,
    is_complete: bool,
}

#[ic_cdk::query]
fn get_client(id: u64) -> Result<Client, Error> {
    match _get_client(&id) {
        Some(client) => Ok(client),
        None => Err(Error::NotFound {
            msg: format!("client id:{} does not exist", id),
        }),
    }
}

// Client

#[ic_cdk::update]
fn add_client(payload: ClientPayload) -> Option<Client> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let client = Client {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        order_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    _insert_client(&client);

    Some(client)
}

// Client Helper functions

fn _get_client(id: &u64) -> Option<Client> {
    CLIENT_STORAGE.with(|clients| clients.borrow().get(&id))
}

fn _insert_client(client: &Client) {
    CLIENT_STORAGE.with(|clients| clients.borrow_mut().insert(client.id, client.clone()));
}

// Supplier

#[ic_cdk::query]
fn get_supplier(id: u64) -> Result<Supplier, Error> {
    match _get_supplier(&id) {
        Some(supplier) => Ok(supplier),
        None => Err(Error::NotFound {
            msg: format!("supplier id:{} does not exist", id),
        }),
    }
}

#[ic_cdk::update]
fn add_supplier(payload: SupplierPayload) -> Option<Supplier> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let supplier = Supplier {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        order_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    _insert_supplier(&supplier);

    Some(supplier)
}

// Supplier Helper functions

fn _get_supplier(id: &u64) -> Option<Supplier> {
    SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow().get(&id))
}

fn _insert_supplier(supplier: &Supplier) {
    SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow_mut().insert(supplier.id, supplier.clone()));
}

// Orders

#[ic_cdk::query]
fn get_order(id: u64) -> Result<Order, Error> {
    match _get_order(&id) {
        Some(order) => Ok(order),
        None => Err(Error::NotFound {
            msg: format!("order id:{} does not exist", id),
        }),
    }
}

#[ic_cdk::query]
fn get_orders() -> Result<Vec<Order>, Error> {
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map.into_iter().map(|(_, order)| order).collect();

    if !orders.is_empty() {
        Ok(orders)
    } else {
        Err(Error::NotFound {
            msg: "No incomplete orders available.".to_string(),
        })
    }
}

#[ic_cdk::update]
fn add_order(payload: OrderPayload) -> Option<Order> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    let order = Order {
        id,
        title: payload.title,
        client_id: payload.client_id,
        supplier_id: None,
        products: payload.products,
        is_complete: false,
        created_at: time(),
        updated_at: None,
    };

    _insert_order(&order);

    Some(order)
}

#[ic_cdk::update]
fn add_order_supplier(id: u64, supplier_id: u64) -> Result<Order, Error> {
    match ORDERS.with(|service| service.borrow().get(&id)) {
        Some(mut order) => {
            order.supplier_id = Some(supplier_id);
            order.updated_at = Some(time());

            _insert_order(&order);
            Ok(order)
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update a order with id={}. order not found", id),
        }),
    }
}

#[ic_cdk::update]
fn update_order(id: u64, payload: OrderPayload) -> Option<Order> {
    let order = ORDERS
        .with(|service| service.borrow().get(&id))
        .expect("order does not exist");

    let updated_order = Order {
        id: order.id,
        title: payload.title,
        client_id: payload.client_id,
        supplier_id: Some(payload.supplier_id),
        products: payload.products,
        is_complete: payload.is_complete,
        created_at: order.created_at,
        updated_at: Some(time()),
    };

    _insert_order(&updated_order);

    if payload.is_complete {
        _update_ids(order)
    }

    Some(updated_order)
}

#[ic_cdk::update]
fn delete_order(id: u64) -> Result<Order, Error> {
    match ORDERS.with(|orders| orders.borrow_mut().remove(&id)) {
        Some(order) => Ok(order),
        None => Err(Error::NotFound {
            msg: format!("Order id:{} deletion unsuccesfull. Order Not found", id),
        }),
    }
}

// Order Helper functions

fn _get_order(id: &u64) -> Option<Order> {
    ORDERS.with(|orders| orders.borrow().get(&id))
}

fn _insert_order(order: &Order) {
    ORDERS.with(|orders| orders.borrow_mut().insert(order.id, order.clone()));
}

fn _update_ids(order: Order) {
    CLIENT_STORAGE.with(|clients| {
        let mut client = clients.borrow_mut().get(&order.client_id).unwrap();

        client.order_ids.push(order.id);
    });

    if order.supplier_id.is_some() {
        SUPPLIER_STORAGE.with(|suppliers| {
            let mut supplier = match order.supplier_id {
                Some(supplier_id) => suppliers.borrow_mut().get(&supplier_id).unwrap(),
                None => panic!("Supplier ID is None"),
            };
            supplier.order_ids.push(order.id);
        });
    }
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

// candid generaterator
ic_cdk::export_candid!();
