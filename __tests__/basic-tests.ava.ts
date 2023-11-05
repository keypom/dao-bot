import anyTest, { TestFn } from "ava";
import { NEAR, NearAccount, Worker } from "near-workspaces";
import { CONTRACT_METADATA, LARGE_GAS, WALLET_GAS, claimWithRequiredGas, doesDropExist, doesKeyExist, functionCall, generateKeyPairs } from "./utils/general";
const { readFileSync } = require('fs')
import { AddedDropDetails, EventDetails, ExtDrop, ExtKeyInfo } from "./utils/types";
import { KeyPair } from "@near-js/crypto";

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
    keypomInitialBalance: NEAR;
    keypomInitialStateStaked: NEAR;
}>;

test.beforeEach(async (t) => {
    // Comment this if you want to see console logs
    //console.log = function() {}

    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    // Prepare sandbox for tests, create accounts, deploy contracts, etc.
    const root = worker.rootAccount;

    // Deploy all 3 contracts
    const keypom = await root.devDeploy(`__tests__/ext_wasm/keypom.wasm`);
    await root.deploy(`__tests__/ext_wasm/linkdrop.wasm`);
    const marketplace = await root.devDeploy("./out/access_key_marketplace.wasm")
    
    // Init the 3 contracts
    await root.call(root, 'new', {});
    await keypom.call(keypom, 'new', { root_account: 'test.near', owner_id: keypom.accountId, contract_metadata: CONTRACT_METADATA });
    await marketplace.call(marketplace, 'new', { contract_owner: marketplace.accountId, keypom_contract: keypom.accountId});
    // Test users
    const ali = await root.createSubAccount('ali');
    const funder = await root.createSubAccount('funder');
    const bob = await root.createSubAccount('bob');
    
    await ali.updateAccount({
        amount: NEAR.parse('10000000 N').toString()
    })
    // let keypomBalance = await keypom.balance();
    // console.log('keypom available INITIAL: ', keypomBalance.available.toString())
    // console.log('keypom staked INITIAL: ', keypomBalance.staked.toString())
    // console.log('keypom stateStaked INITIAL: ', keypomBalance.stateStaked.toString())
    // console.log('keypom total INITIAL: ', keypomBalance.total.toString())

    // Save state for test runs
    t.context.worker = worker;
    t.context.accounts = { root, keypom, marketplace, funder, ali, bob };
});

// If the environment is reused, use test.after to replace test.afterEach
test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

const TERA_GAS = 1000000000000;

// Standard add and remove
// test('Creating Event, Adding Drops', async t => {
//     const { keypom, marketplace, funder, ali, bob } = t.context.accounts;
    
//     const event_id = "moon-concert"
//     await functionCall({
//         signer: funder,
//         receiver: marketplace,
//         methodName: 'list_event',
//         args: {
//             event_id,
//             max_markup: 2,
//         },
//         attachedDeposit: NEAR.parse("1").toString()
//     })

//     const dropId_premium = "drop-id-premium";
//     await functionCall({
//         signer: funder,
//         receiver: keypom,
//         methodName: 'create_drop',
//         args: {
//             drop_id: dropId_premium,
//             asset_data: [{
//                 assets: [null],
//                 uses: 2
//             }],
//             key_data: [],
//         },
//         attachedDeposit: NEAR.parse("1").toString()
//     })
    
//     const dropId_normal = "drop-id-normal";
//     await functionCall({
//         signer: funder,
//         receiver: keypom,
//         methodName: 'create_drop',
//         args: {
//             drop_id: dropId_normal,
//             asset_data: [{
//                 assets: [null],
//                 uses: 2
//             }],
//             key_data: [],
//         },
//         attachedDeposit: NEAR.parse("1").toString()
//     })

//     let added_drops: Record<string, AddedDropDetails> = {};

//     added_drops[dropId_normal] = {max_tickets: 20, price_by_drop_id: 2000};
//     added_drops[dropId_premium] = {max_tickets: 5, price_by_drop_id: 10000};
//     await functionCall({
//         signer: funder,
//         receiver: marketplace,
//         methodName: 'add_drop_to_event',
//         args: {
//             event_id,
//             added_drops
//         },
//         attachedDeposit: NEAR.parse("1").toString()
//     })

//     let event: EventDetails = await marketplace.view("get_event_information", {event_id});
//     console.log(event)
// });

test('Buying Tickets Firsthand', async t => {
    // Attempt to buy premium ticket
    const { keypom, marketplace, funder, ali, bob } = t.context.accounts;
    
    const event_id = "moon-concert"
    await functionCall({
        signer: funder,
        receiver: marketplace,
        methodName: 'list_event',
        args: {
            event_id,
            max_markup: 2,
        },
        attachedDeposit: NEAR.parse("1").toString()
    })

    const dropId_premium = "drop-id-premium";
    await functionCall({
        signer: funder,
        receiver: keypom,
        methodName: 'create_drop',
        args: {
            drop_id: dropId_premium,
            asset_data: [{
                assets: [null],
                uses: 2
            }],
            key_data: [],
        },
        attachedDeposit: NEAR.parse("1").toString()
    })
    
    const dropId_normal = "drop-id-normal";
    await functionCall({
        signer: funder,
        receiver: keypom,
        methodName: 'create_drop',
        args: {
            drop_id: dropId_normal,
            asset_data: [{
                assets: [null],
                uses: 2
            }],
            key_data: [],
        },
        attachedDeposit: NEAR.parse("1").toString()
    })

    let added_drops: Record<string, AddedDropDetails> = {};

    added_drops[dropId_normal] = {max_tickets: 20, price_by_drop_id: 2000};
    added_drops[dropId_premium] = {max_tickets: 5, price_by_drop_id: 10000};
    await functionCall({
        signer: funder,
        receiver: marketplace,
        methodName: 'add_drop_to_event',
        args: {
            event_id,
            added_drops
        },
        attachedDeposit: NEAR.parse("1").toString()
    })

    let event: EventDetails = await marketplace.view("get_event_information", {event_id});
    console.log(event)

    let new_keys: { keys: KeyPair[]; publicKeys: string[] } = await generateKeyPairs(1);

    await functionCall({
        signer: ali,
        receiver: marketplace,
        methodName: "buy_initial_sale",
        args: {
            event_id,
            new_key_info: {
                public_key: new_keys.publicKeys[0],
                key_owner: ali.accountId
            },
            // tier 1 = lowest tier, normal
            ticket_tier: 1
        },
        attachedDeposit: NEAR.parse("10000").toString(),
        shouldPanic: true
    })

    let expectedKey = new_keys.publicKeys[0]

    let key_info: ExtKeyInfo = await keypom.view("get_key_information", {expectedKey});
    console.log(key_info)
    t.is(key_info.owner_id == ali.accountId, true);

});

// test('A', async t => {
// console.log("jello")
// });

