# Rust IC Canister: Supplier and Order Management

## Overview

This Rust project implements a canister for managing suppliers, clients, and orders within the context of the Internet Computer (IC). The canister exposes functions for querying and updating data related to clients, suppliers, and orders. It utilizes the `ic_cdk` framework, and the data is stored in a thread-local storage structure (`StableBTreeMap`) managed by a `MemoryManager`.

## Prerequisites

- Rust
- Internet Computer SDK
- IC CDK

## Installation

1. **Clone the repository:**

    ```bash
    git clone https://github.com/your-username/your-repository.git
    cd your-repository
    ```

## Data Structures

The core data structures are defined as Rust structs:

```rust
#[derive(CandidType, etc)]
struct Client {
  id: u64,
  name: String,
  // etc
}

#[derive(CandidType, etc)]  
struct Supplier {
  // fields 
}

#[derive(CandidType, etc)]
struct Order {
  // fields
}
```

Each record contains relevant fields like IDs, names and timestamps.

## Memory Management

Memory is allocated using a `MemoryManager` from the `ic-stable-structures` crate: 

```rust
static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = // initialized
```

This manages allocating `VirtualMemory` for storages.

## ID Generation

Unique IDs are generated using a thread-local `IdCell`:

```rust
static ID_COUNTER: RefCell<IdCell> = // initialized
```

The counter is incremented when adding new records.

## Record Storage

Records are stored in thread-local `StableBTreeMap`s:

```rust
static CLIENT_STORAGE: RefCell<StableBTreeMap<u64, Client>> = // initialized

static SUPPLIER_STORAGE: RefCell<StableBTreeMap<u64, Supplier>> = // initialized

static ORDERS: RefCell<StableBTreeMap<u64, Order>> = // initialized 
```

This provides fast random access to records.

## Traits

The `Storable` and `BoundedStorable` traits are implemented for serialization and bounding record sizes during storage.

## Candid Interface

Query and update functions are defined using attributes:

```rust
#[ic_cdk::query]
fn get_client(id: u64) -> Result<Client, Error> {
  // ...
}

#[ic_cdk::update]
fn add_order(payload: OrderPayload) -> Option<Order> {
  // ...
}
```

This exposes a Candid API.

## Main Functions

`add_client(payload: ClientPayload) -> Option<Client>`  
Creates a new client based on the provided payload and adds it to the client storage. Returns the created client if successful.

`add_supplier(payload: SupplierPayload) -> Option<Supplier>`  
Creates a new supplier based on the provided payload and adds it to the supplier storage. Returns the created supplier if successful.

`add_order(payload: OrderPayload) -> Option<Order>`  
Creates a new order based on the provided payload and adds it to the order storage. Returns the created order if successful.

`add_order_supplier(id: u64, supplier_id: u64) -> Result<Order, Error>`  
Associates a supplier with an existing order identified by the given ID. Returns the updated order if successful, otherwise returns a NotFound error if the order is not found.

`complete_order(id: u64) -> Result<Order, Error>`  
Marks an order as complete based on the provided ID. Returns the completed order if successful, otherwise returns a NotFound error if the order is not found.

`update_order(id: u64, payload: OrderPayload) -> Option<Order>`  
Updates the information of an existing order identified by the given ID with the provided payload. Returns the updated order if successful.

`delete_order(id: u64) -> Result<Order, Error>`  
Deletes an order based on the provided ID. Returns the deleted order if successful, otherwise returns a NotFound error if the order is not found.

## Helper Functions

Private helper functions handle data access:

```rust 
fn _get_client(id: &u64) -> Option<Client> {
  // lookup client
}

fn _insert_client(client: &Client) {
  // insert into storage
}
```

## Error Handling

The `Error` enum captures errors from record lookups.

This provides a full-featured in-memory database with types and interfaces to manage orders, clients, and suppliers through a thread-safe Rust implementation.

## Usage

This canister provides functionality for managing clients, suppliers, and orders through a set of query and update functions. See the [Functions](#functions) section for details on available operations.

<!-- **Example:** -->

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd supply_chain/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

## Learn More

To learn more before you start working with supply_chain, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)