use near_sdk::PanicOnDefault;

use crate::*;

/// The ID for a given drop (this is the unique identifier for the drop and is how it will be referenced)
pub type DropId = String;
/// Which specific use is something being acted on. This is not zero indexed (i.e the first use is 1)
pub type UseNumber = u32;
/// ID for NFTs that have been sent to the Keypom contract as part of NFT assets
pub type TokenId = String;

/// The ID for a given event (this is the unique identifier for the drop and is how it will be referenced)
pub type EventID = String;


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ResaleConditions {
    // Maximum markup that a ticket can be listed for, as a %
    // For example, 1.2x is 120% -> max_markup = 120
    pub max_markup: u64
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Status {
    Active,
    Inactive
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct EventDetails {
    // Public Facing event name
    pub name: Option<String>,
    // Event hosts, not necessarily the same as all the drop funders
    pub host: Option<AccountId>,
    // Event ID, in case on needing to abstract on contract to multiple drops per event
    // For now, event ID is drop ID
    pub event_id: String,
    // Event Status, can only be active or inactive
    pub status: Status,
    // Description
    pub description: Option<String>,
    // Date
    pub date: Option<String>,
    // Maximum markup, as a %
    pub max_markup: u64,
    // Maximum number of tickets
    pub max_tickets: HashMap<DropId, Option<u64>>,
    // Associated Drop IDs
    // drop - tier link create here, either implicitely through vec or unorderedmap 
    pub drop_ids: Vec<DropId>,
    // Tiered Pricing?
    pub price_by_drop_id: HashMap<DropId, Option<Balance>>,
    
    // Every event should be capable of tiered ticketing, i.e multiple drops per event
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
// For each drop being added to event, the following bits of info are needed.
pub struct AddedDropDetails {
    // Maximum number of tickets
    pub max_tickets: Option<u64>,
    // Tiered Pricing?
    pub price_by_drop_id: Option<Balance>,
    
    // Every event should be capable of tiered ticketing, i.e multiple drops per event
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ExtKeyData {
    /// What is the public key?
    pub public_key: PublicKey,
    /// A map outlining what the password should be for any given use.
    /// The password here should be a double hash and when claim is called,
    /// The user arguments are hashed and compared to the password here (i.e user passes in single hash)
    pub password_by_use: Option<HashMap<UseNumber, String>>,
    /// Metadata for the given key represented as a string. Most often, this will be JSON stringified.
    pub metadata: Option<String>,
    /// What account ID owns the given key (if any)
    pub key_owner: Option<AccountId>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtKeyInfo {
    /// How much Gas should be attached when the key is used to call `claim` or `create_account_and_claim`.
   /// It is up to the smart contract developer to calculate the required gas (which can be done either automatically on the contract or on the client-side).
   pub required_gas: String,

   /// yoctoNEAR$ amount that will be sent to the account that claims the linkdrop (either new or existing)
   /// when the key is successfully used.
   pub yoctonear: U128,

   /// If using the FT standard extension, a set of FTData can be linked to the public key
   /// indicating that all those assets will be sent to the account that claims the linkdrop (either new or
   /// existing) when the key is successfully used.
   pub ft_list: Vec<FTListData>, 
   
   /// If using the NFT standard extension, a set of NFTData can be linked to the public key
   /// indicating that all those assets will be sent to the account that claims the linkdrop (either new or
   /// existing) when the key is successfully used.
   pub nft_list: Vec<NFTListData>, 

   /* CUSTOM */
   pub drop_id: DropId,
   pub pub_key: PublicKey,
   pub token_id: TokenId,
   pub owner_id: AccountId,
   pub fc_list: Vec<FCData>,
   
   pub uses_remaining: UseNumber
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PanicOnDefault, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct FCData {
    pub methods: Vec<MethodData>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MethodData {
    /// Contract that will be called
    pub receiver_id: String,
    /// Method to call on receiver_id contract
    pub method_name: String,
    /// Arguments to pass in (stringified JSON)
    pub args: String,
    /// Amount of yoctoNEAR to attach along with the call
    pub attached_deposit: U128,
    /// How much gas to attach to this method call.
    pub attached_gas: Gas,

    /// Keypom Args struct to be sent to external contracts
    pub keypom_args: Option<KeypomInjectedArgs>,
    /// If set to true, the claiming account ID will be the receiver ID of the method call.
    /// Ths receiver must be a valid account and non-malicious (cannot be set to the keypom contract) 
    pub receiver_to_claimer: Option<bool>,
    /// What permissions does the user have when providing custom arguments to the function call?
    /// By default, the user cannot provide any custom arguments
    pub user_args_rule: Option<UserArgsRule>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtNFTKey {
    //token ID
    pub token_id: TokenId,
    //owner of the token
    pub owner_id: AccountId,
    //token metadata
    pub metadata: TokenMetadata,
    //list of approved account IDs that have access to transfer the token. This maps an account ID to an approval ID
    pub approved_account_ids: HashMap<AccountId, u64>,
    //keep track of the royalty percentages for the token in a hash map
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<u64>, // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>, // When token expires, Unix epoch in milliseconds
    pub starts_at: Option<u64>, // When token starts being valid, Unix epoch in milliseconds
    pub updated_at: Option<u64>, // When token was last updated, Unix epoch in milliseconds
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Injected Keypom Args struct to be sent to external contracts
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct KeypomInjectedArgs {
    /// Specifies what field the claiming account ID should go in when calling the function
    /// If None, this isn't attached to the args
    pub account_id_field: Option<String>,
    /// Specifies what field the drop ID should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    /// If Some(String), attach drop ID to args. Else, don't attach.
    pub drop_id_field: Option<String>,
    /// Specifies what field the key ID should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    /// If Some(String), attach key ID to args. Else, don't attach.
    pub key_id_field: Option<String>,
    // Specifies what field the funder id should go in when calling the function. To insert into nested objects, use periods to separate. For example, to insert into args.metadata.field, you would specify "metadata.field"
    // If Some(string), attach the funder ID to the args. Else, don't attach.
    pub funder_id_field: Option<String>,
}

/// Data outlining Fungible Tokens that should be sent to the claiming account
/// (either new or existing) when a key is successfully used.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct FTListData {
    /// The number of tokens to transfer, wrapped in quotes and treated
    /// like a string, although the number will be stored as an unsigned integer
    /// with 128 bits.
    pub amount: String,

    /// The valid NEAR account indicating the Fungible Token contract.
    pub contract_id: String
}


/// Data outlining a specific Non-Fungible Token that should be sent to the claiming account
/// (either new or existing) when a key is successfully used.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTListData {
    /// the id of the token to transfer
    pub token_id: String,

    /// The valid NEAR account indicating the Non-Fungible Token contract.
    pub contract_id: String
}

#[derive(BorshSerialize, BorshDeserialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
/// When a user provides arguments for FC drops in `claim` or `create_account_and_claim`, what behaviour is expected?
/// For `AllUser`, any arguments provided by the user will completely overwrite any previous args provided by the drop creator.
/// For `FunderPreferred`, any arguments provided by the user will be concatenated with the arguments provided by the drop creator. If there are any duplicate args, the drop funder's arguments will be used.
/// For `UserPreferred`, any arguments provided by the user will be concatenated with the arguments provided by the drop creator, but if there are any duplicate keys, the user's arguments will overwrite the drop funder's.
pub enum UserArgsRule {
    AllUser,
    FunderPreferred,
    UserPreferred
}