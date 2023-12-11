#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::collections::HashMap;
use std::{borrow::Cow, cell::RefCell};
use validator::Validate;

// Define type aliases for convenience
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Define a struct for the 'Client'
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

// Define a struct for the 'Supplier'
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Supplier {
    id: u64,
    name: String,
    email: String,
    phone: String,
    prefered_items: Vec<String>,
    order_ids: Vec<u64>,
    created_at: u64,
    updated_at: Option<u64>,
}

// Define a struct for the 'Order'
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Order {
    id: u64,
    title: String,
    client_id: u64,
    supplier_id: Option<u64>,
    item_types: Vec<String>,
    products: HashMap<String, u64>,
    is_complete: bool,
    created_at: u64,
    updated_at: Option<u64>,
}

// Implement the 'Storable' trait for 'Client', 'Supplier', and 'Order'
impl Storable for Client {
    // Conversion to bytes
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    // Conversion from bytes
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

// Implement the 'BoundedStorable' trait for 'Client', 'Supplier', and 'Order'
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

// Define thread-local static variables for memory management and storage
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

// Define structs for payload data (used in update calls)
#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct ClientPayload {
    #[validate(length(min = 3))]
    name: String,
    email: String,
    // shortest international phone number possible is 7 digits and longest possible is 15 digits(international phone numbering plan (ITU-T E. 164))
    #[validate(length(min = 7, max = 15))]
    phone: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct SupplierPayload {
    #[validate(length(min = 3))]
    name: String,
    email: String,
    // shortest international phone number possible is 7 digits and longest possible is 15 digits(international phone numbering plan (ITU-T E. 164))
    #[validate(length(min = 7, max = 15))]
    phone: String,
    prefered_items: Vec<String>,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Validate)]
struct OrderPayload {
    #[validate(length(min = 1))]
    title: String,
    client_id: u64,
    supplier_id: u64,
    products: HashMap<String, u64>,
    items_types: Vec<String>,
    is_complete: bool,
}

#[derive(candid::CandidType, Deserialize, Serialize, Default)]
struct AddOrderSupplierPayload {
    order_id: u64,
    supplier_id: u64,
}

// Define query function to get a client by ID
#[ic_cdk::query]
fn get_client(id: u64) -> Result<Client, Error> {
    match _get_client(&id) {
        Some(client) => Ok(client),
        None => Err(Error::NotFound {
            msg: format!("client id:{} does not exist", id),
        }),
    }
}

// Update function to add a client
#[ic_cdk::update]
fn add_client(payload: ClientPayload) -> Result<Client, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: check_payload.unwrap_err().to_string(),
        });
    }
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

    Ok(client)
}

// Helper function to get a client by ID
fn _get_client(id: &u64) -> Option<Client> {
    CLIENT_STORAGE.with(|clients| clients.borrow().get(&id))
}

// Helper function to insert a client
fn _insert_client(client: &Client) {
    CLIENT_STORAGE.with(|clients| clients.borrow_mut().insert(client.id, client.clone()));
}

// Supplier
#[ic_cdk::query]
fn get_supplier(id: u64) -> Result<Supplier, Error> {
    // Try to get the supplier with the given id
    match _get_supplier(&id) {
        Some(supplier) => Ok(supplier), // Return the supplier if found
        None => Err(Error::NotFound {
            msg: format!("supplier id:{} does not exist", id),
        }), // Return an error if the supplier is not found
    }
}

#[ic_cdk::query]
fn get_suppliers() -> Result<Vec<Supplier>, Error> {
    // Retrieve all suppliers from the storage
    let suppliers_map: Vec<(u64, Supplier)> =
        SUPPLIER_STORAGE.with(|service| service.borrow().iter().collect());
    let suppliers: Vec<Supplier> = suppliers_map
        .into_iter()
        .map(|(_, supplier)| supplier)
        .collect();

    if !suppliers.is_empty() {
        Ok(suppliers) // Return the list of suppliers if not empty
    } else {
        Err(Error::NotFound {
            msg: "No suppliers available.".to_string(),
        }) // Return an error if no suppliers are found
    }
}

#[ic_cdk::update]
fn add_supplier(payload: SupplierPayload) -> Result<Supplier, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: check_payload.unwrap_err().to_string(),
        });
    }
    // Increment the global ID counter to get a new ID for the supplier
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new Supplier with the provided payload and the generated ID
    let supplier = Supplier {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        prefered_items: payload.prefered_items,
        order_ids: vec![],
        created_at: time(),
        updated_at: None,
    };

    // Insert the new supplier into the storage
    _insert_supplier(&supplier);

    Ok(supplier) // Return the newly added supplier
}

// Supplier Helper functions

fn _get_supplier(id: &u64) -> Option<Supplier> {
    // Get the supplier from the storage based on the provided ID
    SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow().get(&id))
}

fn _insert_supplier(supplier: &Supplier) {
    // Insert a supplier into the storage
    SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow_mut().insert(supplier.id, supplier.clone()));
}

// Orders

#[ic_cdk::query]
fn get_order(id: u64) -> Result<Order, Error> {
    // Try to get the order with the given ID
    match _get_order(&id) {
        Some(order) => Ok(order), // Return the order if found
        None => Err(Error::NotFound {
            msg: format!("order id:{} does not exist", id),
        }), // Return an error if the order is not found
    }
}

#[ic_cdk::query]
fn get_orders() -> Result<Vec<Order>, Error> {
    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map.into_iter().map(|(_, order)| order).collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: "No orders available.".to_string(),
        }) // Return an error if no orders are found
    }
}

#[ic_cdk::query]
fn get_incomplete_orders() -> Result<Vec<Order>, Error> {
    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| !order.is_complete)
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: "No incomplete orders available.".to_string(),
        }) // Return an error if no orders are found
    }
}

#[ic_cdk::query]
fn get_supplier_preferred_orders(supplier_id: u64) -> Result<Vec<Order>, Error> {
    if !is_supplier_id_valid(&supplier_id) {
        return Err(Error::NotFound {
            msg: format!("Supplier with id={} not found.", supplier_id),
        });
    }
    // Retrieve the supplier's preferred items
    let preferred_items = _get_supplier_preferred_items(supplier_id);

    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());

    // Filter the orders to only include those where the order_types match the supplier's preferred_items
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| {
            preferred_items
                .iter()
                .any(|item| order.item_types.contains(item))
        })
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: format!(
                "No orders available matching supplier id:{} prefferences",
                supplier_id
            ),
        }) // Return an error if no orders are found
    }
}

fn _get_supplier_preferred_items(supplier_id: u64) -> Vec<String> {
    // Retrieve the supplier from the storage
    let supplier = SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow().get(&supplier_id));

    // Return the supplier's preferred items if the supplier is found
    if let Some(supplier) = supplier {
        supplier.prefered_items
    } else {
        vec![] // Return an empty vector if the supplier is not found
    }
}

#[ic_cdk::query]
fn get_completed_orders() -> Result<Vec<Order>, Error> {
    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| order.is_complete)
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: "No completed orders available.".to_string(),
        }) // Return an error if no orders are found
    }
}

#[ic_cdk::query]
fn get_client_orders(client_id: u64) -> Result<Vec<Order>, Error> {
    if !is_client_id_valid(&client_id) {
        return Err(Error::NotFound {
            msg: format!("Client with id={} not found.", client_id),
        });
    }

    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| order.client_id == client_id)
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: format!("No orders available for client id:{}", client_id),
        }) // Return an error if no orders are found
    }
}

#[ic_cdk::query]
fn get_supplier_orders(supplier_id: u64) -> Result<Vec<Order>, Error> {
    if !is_supplier_id_valid(&supplier_id) {
        return Err(Error::NotFound {
            msg: format!("Supplier with id={} not found.", supplier_id),
        });
    }
    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| order.supplier_id == Some(supplier_id))
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: format!("No orders available for supplier id:{}", supplier_id),
        }) // Return an error if no orders are found
    }
}

#[ic_cdk::query]
fn get_supplier_completed_orders(supplier_id: u64) -> Result<Vec<Order>, Error> {
    if !is_supplier_id_valid(&supplier_id) {
        return Err(Error::NotFound {
            msg: format!("Supplier with id={} not found.", supplier_id),
        });
    }
    // Retrieve all orders from the storage
    let orders_map: Vec<(u64, Order)> = ORDERS.with(|service| service.borrow().iter().collect());
    let orders: Vec<Order> = orders_map
        .into_iter()
        .map(|(_, order)| order)
        .filter(|order| order.supplier_id == Some(supplier_id) && order.is_complete)
        .collect();

    if !orders.is_empty() {
        Ok(orders) // Return the list of orders if not empty
    } else {
        Err(Error::NotFound {
            msg: format!(
                "No completed orders available for supplier id:{}",
                supplier_id
            ),
        }) // Return an error if no orders are found
    }
}
// Function to create an order
#[ic_cdk::update]
fn add_order(payload: OrderPayload) -> Result<Order, Error> {
    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: check_payload.unwrap_err().to_string(),
        });
    }
    if !is_client_id_valid(&payload.client_id) {
        return Err(Error::NotFound {
            msg: format!("Client with id={} not found.", payload.client_id),
        });
    }
    // Increment the global ID counter to get a new ID for the order
    let id = ID_COUNTER
        .with(|counter| {
            let current_id = *counter.borrow().get();
            counter.borrow_mut().set(current_id + 1)
        })
        .expect("Cannot increment Ids");

    // Create a new Order with the provided payload and the generated ID
    let order = Order {
        id,
        title: payload.title,
        client_id: payload.client_id,
        supplier_id: None,
        products: payload.products,
        item_types: payload.items_types,
        is_complete: false,
        created_at: time(),
        updated_at: None,
    };

    // Insert the new order into the storage
    _insert_order(&order);

    Ok(order) // Return the newly added order
}

// Function to add a suplier for an order
#[ic_cdk::update]
fn add_order_supplier(payload: AddOrderSupplierPayload) -> Result<Order, Error> {
    // Try to get the order with the given ID
    match ORDERS.with(|service| service.borrow().get(&payload.order_id)) {
        Some(mut order) => {
            if !is_supplier_id_valid(&payload.supplier_id) {
                return Err(Error::NotFound {
                    msg: format!("Supplier with id={} not found.", payload.supplier_id),
                });
            }
            // Update the order with the supplied supplier ID and timestamp
            order.supplier_id = Some(payload.supplier_id);
            order.updated_at = Some(time());

            // Insert the updated order back into the storage
            _insert_order(&order);

            Ok(order) // Return the updated order
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update an order with id={}. order not found",
                payload.order_id
            ),
        }), // Return an error if the order is not found
    }
}
// Function to complete an order
#[ic_cdk::update]
fn complete_order(id: u64) -> Result<Order, Error> {
    // Try to get the order with the given ID
    match ORDERS.with(|service| service.borrow().get(&id)) {
        Some(mut order) => {
            if order.is_complete {
                return Err(Error::AlreadyCompleted {
                    msg: format!("Order was already completed."),
                });
            }
            if !is_client_id_valid(&order.client_id) {
                return Err(Error::NotFound {
                    msg: format!("Client with id={} not found.", order.client_id),
                });
            }
            if order.supplier_id.is_none() {
                return Err(Error::NotFound {
                    msg: format!("No Supplier set for this order."),
                });
            }
            if !is_supplier_id_valid(&order.supplier_id.unwrap()) {
                return Err(Error::NotFound {
                    msg: format!("Supplier with id={} not found.", order.supplier_id.unwrap()),
                });
            }
            // Mark the order as complete and update the timestamp
            order.is_complete = true;
            order.updated_at = Some(time());

            // Insert the updated order back into the storage
            _insert_order(&order);

            if order.is_complete {
                _update_ids(order.clone()) // Update IDs if the order is marked as complete
            }
            Ok(order) // Return the completed order
        }
        None => Err(Error::NotFound {
            msg: format!("couldn't update an order with id={}. order not found", id),
        }), // Return an error if the order is not found
    }
}
// Function to update an order
#[ic_cdk::update]
fn update_order(id: u64, payload: OrderPayload) -> Result<Order, Error> {
    // Try to get the existing order with the given ID
    let order = ORDERS
        .with(|service| service.borrow().get(&id))
        .expect("order does not exist");

    let check_payload = payload.validate();
    if check_payload.is_err() {
        return Err(Error::InvalidPayload {
            msg: check_payload.unwrap_err().to_string(),
        });
    }

    if !is_client_id_valid(&payload.client_id) {
        return Err(Error::NotFound {
            msg: format!("Client with id={} not found.", payload.client_id),
        });
    }
    if !is_supplier_id_valid(&payload.supplier_id) {
        return Err(Error::NotFound {
            msg: format!("Supplier with id={} not found.", payload.supplier_id),
        });
    }

    // Create an updated order based on the provided payload
    let updated_order = Order {
        id: order.id,
        title: payload.title,
        client_id: payload.client_id,
        supplier_id: Some(payload.supplier_id),
        item_types: payload.items_types,
        products: payload.products,
        is_complete: order.is_complete,
        created_at: order.created_at,
        updated_at: Some(time()),
    };

    // Insert the updated order into the storage
    _insert_order(&updated_order);

    Ok(updated_order) // Return the updated order
}

#[ic_cdk::update]
fn delete_order(id: u64) -> Result<Order, Error> {
    // Remove the order with the given ID from the storage
    match ORDERS.with(|orders| orders.borrow_mut().remove(&id)) {
        Some(order) => Ok(order), // Return the deleted order
        None => Err(Error::NotFound {
            msg: format!("Order id:{} deletion unsuccessful. Order Not found", id),
        }), // Return an error if the order is not found
    }
}

// Order Helper functions

fn _get_order(id: &u64) -> Option<Order> {
    // Get the order from the storage based on the provided ID
    ORDERS.with(|orders| orders.borrow().get(&id))
}

fn _insert_order(order: &Order) {
    // Insert an order into the storage
    ORDERS.with(|orders| orders.borrow_mut().insert(order.id, order.clone()));
}

fn _update_ids(order: Order) {
    // Checks were already made to check whether client exists
    let mut client = get_client(order.client_id).ok().unwrap();
    client.order_ids.push(order.id);
    // Update the client's order IDs
    CLIENT_STORAGE.with(|clients| clients.borrow_mut().insert(client.id, client.clone()));

    // Checks were already made to check whether supplier exists
    let mut supplier = get_supplier(order.supplier_id.unwrap()).ok().unwrap();
    supplier.order_ids.push(order.id);
    // Update the supplier's order IDs
    SUPPLIER_STORAGE.with(|suppliers| suppliers.borrow_mut().insert(supplier.id, supplier.clone()));
}

// Helper function to check whether a client with client_id exists
fn is_client_id_valid(client_id: &u64) -> bool {
    let client = get_client(client_id.clone());
    if client.is_err() {
        false
    } else {
        true
    }
}
// Helper function to check whether a supplier with supplier_id exists
fn is_supplier_id_valid(supplier_id: &u64) -> bool {
    let supplier = get_supplier(supplier_id.clone());
    if supplier.is_err() {
        false
    } else {
        true
    }
}
// Define an Error enum for handling errors
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidPayload { msg: String },
    AlreadyCompleted { msg: String}
}

// Candid generator for exporting the Candid interface
ic_cdk::export_candid!();
