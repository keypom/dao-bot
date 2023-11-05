const path = require("path");
const homedir = require("os").homedir();
const { UnencryptedFileSystemKeyStore } = require("@near-js/keystores-node");
const { Account } = require("@near-js/accounts");
const { parseNearAmount } = require("@near-js/utils");
const { Near } = require("@near-js/wallet-account");


const KEYPOM_CONTRACT = "ncon23.keypom.testnet"
const MARKETPLACE = "dev-1682481900002-78543887295736"


async function main(){

    // Change this to your account ID
    const FUNDER_ACCOUNT_ID = "minqi.testnet";
    const NETWORK_ID = "testnet";

    // Initiate connection to the NEAR blockchain.
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath =  path.join(homedir, CREDENTIALS_DIR);

    let keyStore = new UnencryptedFileSystemKeyStore(credentialsPath);

    let nearConfig = {
        networkId: NETWORK_ID,
        keyStore: keyStore,
        nodeUrl: `https://rpc.${NETWORK_ID}.near.org`,
        walletUrl: `https://wallet.${NETWORK_ID}.near.org`,
        helperUrl: `https://helper.${NETWORK_ID}.near.org`,
        explorerUrl: `https://explorer.${NETWORK_ID}.near.org`,
    }; 

    let near = new Near(nearConfig);
    const fundingAccount = new Account(near.connection, FUNDER_ACCOUNT_ID)

    
    let numKeys = 1
    const TERA_GAS = 1000000000000;
    const MAX_GAS = 300 * TERA_GAS

    await fundingAccount.functionCall({
        contractId: MARKETPLACE,
        methodName: "new",
        args: {
            contract_owner: MARKETPLACE.accountId,
            keypom_contract: KEYPOM_CONTRACT.accountId
        },
        gas: MAX_GAS,
        attachedDeposit: parseNearAmount("1")
    })

    // let event_id = "moon-party"
    // try{
    //     await fundingAccount.functionCall({
    //         contractId: MARKETPLACE,
    //         methodName: "list_event",
    //         args: {
    //             // New account ID from user input
    //             event_id,
    //             max_markup: 2
    //         },
    //         gas: MAX_GAS,
    //     })
    // }catch(e){
    //     console.log(e)
    // }

    // const dropId_premium = "drop-id-premium";
    // try{
    //     await fundingAccount.functionCall({
    //         contractId: KEYPOM_CONTRACT,
    //         methodName: "create_drop",
    //         args: {
    //             drop_id: dropId_premium,
    //             asset_data: [{
    //                 assets: [null],
    //                 uses: 2
    //             }],
    //             key_data: [],
    //         },
    //         gas: MAX_GAS,
    //         attachedDeposit: parseNearAmount("1")
    //     })
    // }catch(e){
    //     console.log(e)
    // }

    // const dropId_normal = "drop-id-premium";
    // try{
    //     await fundingAccount.functionCall({
    //         contractId: KEYPOM_CONTRACT,
    //         methodName: "create_drop",
    //         args: {
    //             drop_id: dropId_normal,
    //             asset_data: [{
    //                 assets: [null],
    //                 uses: 2
    //             }],
    //             key_data: [],
    //         },
    //         gas: MAX_GAS,
    //         attachedDeposit: parseNearAmount("1")
    //     })
    // }catch(e){
    //     console.log(e)
    // }

    // try{
    //     await fundingAccount.functionCall({
    //         contractId: MARKETPLACE,
    //         methodName: "add_drop_to_event",
    //         args: {
    //             drop_id: dropId_normal,
    //             asset_data: [{
    //                 assets: [null],
    //                 uses: 2
    //             }],
    //             key_data: [],
    //         },
    //         gas: MAX_GAS,
    //         attachedDeposit: parseNearAmount("1")
    //     })
    // }catch(e){
    //     console.log(e)
    // }

    // await functionCall({
    //     signer: funder,
    //     receiver: marketplace,
    //     methodName: 'add_drop_to_event',
    //     args: {
    //         event_id,
    //         added_drops
    //     },
    //     attachedDeposit: NEAR.parse("1").toString()
    // })
}


main()



module.exports = {
    main,
}
