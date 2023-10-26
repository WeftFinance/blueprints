# Rust Price Feed Module

  

This is a Rust module for a price feed that manages and provides access to price information for various resources. The module also includes role-based access control for admin and updater roles, allowing authorized users to manage and update price data. It includes the following components:

  

-  **PriceInfo**: A struct to represent price information with a timestamp and a decimal value.

  

-  **AuthBadgeData**: A struct used for authorization purposes. It doesn't contain any specific data.

  

-  **UpdaterBadgeData**: A struct for managing updater badges, including an "active" flag.

  

-  **PriceFeed**: The main module that allows you to interact with the price feed. It includes methods for minting updater badges, updating badge data, and managing price information. There are different roles, including admin and updater, with associated methods and access control restrictions.

  
  


### PriceFeed Struct

  

The `PriceFeed` struct contains the following fields:

  

-  `prices`: An `IndexMap` that stores price information for different resources.

-  `updater_badge_manager`: A `ResourceManager` for managing updater badges.

-  `updater_counter`: A counter for generating unique IDs for updater badges.

  

#### Instantiate

  

You can create an instance of the `PriceFeed` module using the `instantiate` function:

  

```rust

pub  fn  instantiate() -> NonFungibleBucket {

// ... Initialization and setup logic

}

```

  

This function initializes the price feed and sets up the necessary rules and roles.

  

### Admin Methods

  

The admin methods are used for managing the price feed and updater badges.

  

#### Mint Updater Badge

  

This method allows the admin to mint updater badges:

  

```rust

pub  fn  mint_updater_badge(&mut  self, active: bool) -> Bucket {

// ... Minting logic

}

```

  

#### Update Updater Badge

  

The admin can update the "active" flag of updater badges using this method:

  

```rust

pub  fn  update_updater_badge(&self, local_id: NonFungibleLocalId, active: bool) {

// ... Updater badge update logic

}

```

  

#### Admin Update Price

  

This method allows the admin to update price information for resources:

  

```rust

pub  fn  admin_update_price(&mut  self, resource: ResourceAddress, price: Decimal) {

// ... Price update logic

}

```

  

### Updater Methods

  

Updater methods are used by updaters to manage price information.

  

#### Update Price

  

Updaters can use this method to update price information for resources, provided they have an active updater badge:

  

```rust

pub  fn  update_price(

&mut  self,

badge_proof: Proof,

resource: ResourceAddress,

price: Decimal,

) {

// ... Price update logic with updater badge verification

}

```

  

### Public Methods

  

Public methods are accessible by anyone and allow access to price information.

  

#### Get Price

  

This method retrieves price information for a specific resource:

  

```rust

pub  fn  get_price(&self, quote: ResourceAddress) -> Option<PriceInfo> {

// ... Price retrieval logic

}

```