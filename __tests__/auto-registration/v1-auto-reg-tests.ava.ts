import anyTest, { TestFn } from "ava";
import { BN, NEAR, NearAccount, Worker, getNetworkFromEnv } from "near-workspaces";
import { CONTRACT_METADATA, displayFailureLog, generateKeyPairs, LARGE_GAS, queryAllViewFunctions, WALLET_GAS } from "../utils/general";
import { DropConfig, FCData } from "../utils/types";



type kind = {
    Group?: String[];
}


type role = {
    name: string;
    kind: kind;
};

type policy = {
    roles: role[];
}

// Parsing user roles
function getUserRoles (policyInfo: policy, accountId: string) {
    let roles: string[] = [];
    roles.push('All')

    // Loop through each element in res.roles
    for (const role of policyInfo.roles) {
        const roleKind = role.kind;
        console.log('roleKind: ', roleKind)
        const roleName = role.name;
        console.log('roleName: ', roleName)

        
        console.log(roleKind.Group?.toString());
        if (roleKind.Group?.includes(accountId)){
            roles.push(roleName)
        }
    }

    return roles
}

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

    // Creating dao member accounts
    const minqi = await root.createSubAccount('minqi');
    const member1 = await root.createSubAccount('member1');
    const maliciousActor = await root.createSubAccount('maliciousactor');

    // Deploy all 3 contracts
    const keypom = await root.devDeploy(`./__tests__/ext_wasm/keypom.wasm`);
    const dao = await root.devDeploy(`./__tests__/ext_wasm/sputnikdao2.wasm`);
    const daoMalicious = await root.devDeploy(`./__tests__/ext_wasm/sputnikdao2.wasm`);
    const daoBot = await root.devDeploy(`./out/dao_bot_v1.wasm`);

    console.log(`KEYPOM: ${keypom.accountId}`);
    console.log(`DAO: ${dao.accountId}`);
    console.log(`DAO_BOT: ${daoBot.accountId}`);

    // Init the dao, Keypom, and daobot contracts
    await dao.call(dao, 'new', 
    {
        config: {
            name: 'keypomtestdao', 
            purpose: 'to test adding members automatically', 
            metadata: ''
        }, 
        policy: [minqi.accountId]
    })

    await keypom.call(keypom, 'new', { root_account: 'test.near', owner_id: keypom, contract_metadata: CONTRACT_METADATA });



    // Add daoBot as its own role
    let daobot_proposal_id = await minqi.call(dao, 'add_proposal', 
        {   proposal: {
                description: 'adding daobot', 
                kind: {
                    ChangePolicyAddOrUpdateRole: {
                        role: {
                            name: 'keypom-daobot', 
                            kind: { Group: [daoBot.accountId]},
                            permissions: ['*:*'],
                            vote_policy: {}
                        }
                    }
                },
            }
        },
        {gas: new BN(30 * 10**12),
        attachedDeposit: NEAR.parse("0.1").toString()}
    );
    
    await minqi.call(dao, 'act_proposal',
    {
        id: daobot_proposal_id,
        action: 'VoteApprove'
    })

    // Add new-onboardee-role
    let onboardee_proposal_id = await minqi.call(dao, 'add_proposal', 
        {   proposal: {
                description: 'adding onboardee role', 
                kind: {
                    ChangePolicyAddOrUpdateRole: {
                        role: {
                            name: 'new-onboardee-role', 
                            kind: { Group: [minqi.accountId]},
                            permissions: ['*:AddProposal'],
                            vote_policy: {}
                        }
                    }
                },
            }
        },
        {gas: new BN(30 * 10**12),
        attachedDeposit: NEAR.parse("0.1").toString()}
    );
    
    await minqi.call(dao, 'act_proposal',
    {
        id: onboardee_proposal_id,
        action: 'VoteApprove'
    })
    
    let keypomBalance = await keypom.balance();
    console.log('keypom available INITIAL: ', keypomBalance.available.toString())
    console.log('keypom staked INITIAL: ', keypomBalance.staked.toString())
    console.log('keypom stateStaked INITIAL: ', keypomBalance.stateStaked.toString())
    console.log('keypom total INITIAL: ', keypomBalance.total.toString())

    // Save state for test runs
    t.context.worker = worker;
    t.context.accounts = { root, keypom, dao, daoMalicious, daoBot, minqi, member1, maliciousActor };
});

// If the environment is reused, use test.after to replace test.afterEach
test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

// PURPOSE: Ensure malicious actors with their own daos cannot gain access to someone else's dao
test('Malicious Actors with their own DAOs', async t => {
    const { keypom, dao, daoMalicious, daoBot, minqi, member1, maliciousActor } = t.context.accounts;

    // Set up daoMalicious, maliciousActor's mock dao
    await maliciousActor.call(daoMalicious, 'new', 
    {
        config: {
            name: 'keypomtestdao', 
            purpose: 'to test adding members automatically', 
            metadata: ''
        }, 
        policy: [maliciousActor.accountId]
    })

    // Create malicious FC drop attempting to gain access to dao. None of these should work
    const fcData: FCData = {
        methods: [
            // method 1, fake keypom args
            [
                {
                    receiver_id: daoBot.accountId,
                    method_name: "new_proposal",
                    args: JSON.stringify({
                        dao_contract: dao.accountId,
                        proposal: {
                            description: "mooooooooon",
                            kind: {
                                AddMemberToRole:{
                                    role: "new-onboardee-role",
                                    member_id: maliciousActor.accountId
                                }
                            }
                        },
                        funder: minqi.accountId,
                        keypom_args:{
                            funder_id_field: "funder",
                            account_id_field: "proposal.kind.AddMemberToRole.member_id",
                        }
                    }),
                    attached_deposit: NEAR.parse("0.1").toString()
                }
            ],
            // method 2, let keypom populate keypom args but with existing values
            [
                {
                    receiver_id: daoBot.accountId,
                    method_name: "new_proposal",
                    args: JSON.stringify({
                        dao_contract: dao.accountId,
                        proposal: {
                            description: "mooooooooon",
                            kind: {
                                AddMemberToRole:{
                                    role: "new-onboardee-role",
                                    member_id: maliciousActor.accountId
                                }
                            }
                        },
                        funder: minqi.accountId,
                    }),
                    funder_id_field: "funder",
                    account_id_field: "proposal.kind.AddMemberToRole.member_id",
                    attached_deposit: NEAR.parse("0.1").toString()
                }
            ],
        ]   
    }   

    const config: DropConfig = { 
        uses_per_key: 2
    }

    // This should not work
    let {keys, publicKeys} = await generateKeyPairs(1);
    await maliciousActor.call(keypom, 'create_drop', {public_keys: publicKeys, deposit_per_use: NEAR.parse('1').toString(), fc: fcData, config}, {gas: LARGE_GAS, attachedDeposit: NEAR.parse('5.5').toString()});
    
    // claim both uses to test both methods
    await keypom.setKey(keys[0]);
    await keypom.call(keypom, 'claim', {account_id: member1.accountId}, {gas: WALLET_GAS});
    await keypom.call(keypom, 'claim', {account_id: member1.accountId}, {gas: WALLET_GAS});
    
    // Ensure DAO does not have member1 as an onboardee
    let pol: policy = await dao.view('get_policy');
    
    let member1dao_groups: Array<String> = getUserRoles(pol, member1.accountId);
    t.is(member1dao_groups.includes('new-onboardee-role'), false);
    // t.is(1==1, true);
});

// // PURPOSE: Normal claiming process
test('Normal Claiming Process', async t => {
    const { keypom, dao, daoBot, minqi, member1, maliciousActor, member3 } = t.context.accounts;

    const fcData: FCData = {
        methods: [
            [
                {
                    receiver_id: daoBot.accountId,
                    method_name: "new_proposal",
                    args: JSON.stringify({
                        dao_contract: dao.accountId,
                        proposal: {
                            description: "mooooooooon",
                            kind: {
                                AddMemberToRole:{
                                    role: "new-onboardee-role",
                                }
                            }
                        },
                    }),
                    funder_id_field: "funder",
                    account_id_field: "proposal.kind.AddMemberToRole.member_id",
                    attached_deposit: NEAR.parse("0.1").toString()
                }
            ],
        ]   
    }   

    const config: DropConfig = { 
        uses_per_key: 1
    }

    let {keys, publicKeys} = await generateKeyPairs(1);
    await minqi.call(keypom, 'create_drop', {public_keys: publicKeys, deposit_per_use: NEAR.parse('1').toString(), fc: fcData, config}, {gas: LARGE_GAS, attachedDeposit: NEAR.parse('3').toString()});
    
    await keypom.setKey(keys[0]);
    await keypom.call(keypom, 'claim', {account_id: member1.accountId}, {gas: WALLET_GAS});
    
    let pol: policy = await dao.view('get_policy');
    let member1_groups: Array<String> = getUserRoles(pol, member1.accountId);    
    t.is(member1_groups.includes('new-onboardee-role'), true);
    // t.is(1==1, true);
});

// // PURPOSE: Ensure Keypom contract stored on DAO bot can be changed but only by daoBot
test('DAO Bot Keypom Contract State Variable Security', async t => {
    const { daoBot, minqi } = t.context.accounts;

    let keypomContract = await daoBot.view("view_keypom_contract");
    t.is(keypomContract == "v2.keypom.testnet", true);

    // This should not work, try catch used to catch error and continue testing
    try{
        await minqi.call(daoBot, "change_keypom_contract", {new_contract: "abc.testnet"})
    }
    catch(err){
        // verify it has not changed
        t.is(keypomContract == "v2.keypom.testnet", true);
    }

    // This should work
    await daoBot.call(daoBot, "change_keypom_contract", {new_contract: "v1-3.keypom.testnet"})
    keypomContract = await daoBot.view("view_keypom_contract");
    t.is(keypomContract == "v1-3.keypom.testnet", true);
});