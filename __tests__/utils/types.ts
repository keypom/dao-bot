export type ExtDrop = {
    assets_by_use: Record<number, Array<ExtAsset>>;
    nft_asset_data: Array<InternalNFTData>;
    ft_asset_data: Array<InternalFTData>;
    drop_config?: DropConfig|null;
}

export type DropConfig = {
    add_key_allowlist?: Array<string>|null,
}

export type AddedDropDetails = {
    // Maximum number of tickets
    max_tickets: number|undefined,
    // Tiered Pricing?
    price_by_drop_id: number|undefined,
    
    // Every event should be capable of tiered ticketing, i.e multiple drops per event
}

export type EventDetails = {
    // Public Facing event name
    name: string|undefined,
    // Event hosts, not necessarily the same as all the drop funders
    host: string|undefined,
    // Event ID, in case on needing to abstract on contract to multiple drops per event
    // For now, event ID is drop ID
    event_id: String,
    // Event Status, can only be active or inactive
    status: string,
    // Description
    description: string|undefined,
    // Date
    date: string|undefined,
    // Maximum markup, as a %
    max_markup: number,
    // Maximum number of tickets
    max_tickets: Record<string, number|undefined>,
    // Associated Drop IDs
    // drop - tier link create here, either implicitely through vec or unorderedmap 
    drop_ids: [string],
    // Tiered Pricing?
    price_by_drop_id: Record<string, number|undefined>
    
    // Every event should be capable of tiered ticketing, i.e multiple drops per event
}

export type UserProvidedFCArgs = Array<AssetSpecificFCArgs>;
export type AssetSpecificFCArgs = Array<string | undefined> | undefined;

export type PickOnly<T, K extends keyof T> =
    Pick<T, K> & { [P in Exclude<keyof T, K>]?: never };
    
export type ExtKeyInfo = {
    /// How much Gas should be attached when the key is used to call `claim` or `create_account_and_claim`.
    /// It is up to the smart contract developer to calculate the required gas (which can be done either automatically on the contract or on the client-side).
    required_gas: string,

    /// yoctoNEAR$ amount that will be sent to the account that claims the linkdrop (either new or existing)
    /// when the key is successfully used.
    yoctonear: string,

    /// If using the FT standard extension, a set of FTData can be linked to the public key
    /// indicating that all those assets will be sent to the account that claims the linkdrop (either new or
    /// existing) when the key is successfully used.
    ft_list: Array<ExtFTData>, 

    /* CUSTOM */
    uses_remaining: Number,
    token_id: string,
    owner_id: string,
}

export type InternalAsset = InternalFTData | InternalNFTData | "near";

export type InternalFTData = {
    contract_id: string;
    registration_cost: string,
    balance_avail: string
}

export type InternalNFTData = {
    contract_id: string;
    token_ids: Array<string>
}

export type TokenMetadata = {
    title: string|undefined,
    description: string,
    media: string,
    media_hash: string|undefined,
    copies: number|undefined,
    issued_at: number|undefined,
    expires_at: number|undefined,
    starts_at: number|undefined,
    updated_at: number|undefined,
    extra: string|undefined,
    reference: string|undefined,
    reference_hash: number[]|undefined
}

export type ExtAsset = ExtFTData;

export type ExtFTData = {
    ft_contract_id: string;
    registration_cost: string,
    ft_amount: string
}

export type ExtNFTData = {
    nft_contract_id: string
}

export type ExtNearData = {
    yoctonear: string
}
