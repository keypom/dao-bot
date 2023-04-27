import anyTest, { TestFn } from "ava";
import { BN, NEAR, NearAccount, Worker, getNetworkFromEnv } from "near-workspaces";
import { CONTRACT_METADATA, displayFailureLog, generateKeyPairs, LARGE_GAS, queryAllViewFunctions, WALLET_GAS } from "../utils/general";
import { DropConfig, FCData } from "../utils/types";

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
    const member2 = await root.createSubAccount('member2');
    const member3 = await root.createSubAccount('member3');

    // Deploy all 3 contracts
    const keypom = await root.devDeploy(`./__tests__/ext_wasm/keypom.wasm`);
    const dao = await root.devDeploy(`./__tests__/ext_wasm/sputnikdao2.wasm`)
    const daoBot = await root.devDeploy(`./target/wasm32-unknown-unknown/release/dao_bot.wasm`)

    console.log(`KEYPOM: ${keypom.accountId}`);
    console.log(`DAO: ${dao.accountId}`);
    console.log(`DAO_BOT: ${daoBot.accountId}`);

    // Init the dao and Keypom contracts
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
        attachedDeposit: NEAR.parse("1").toString()}
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
        attachedDeposit: NEAR.parse("1").toString()}
    );
    
    await minqi.call(dao, 'act_proposal',
    {
        id: onboardee_proposal_id,
        action: 'VoteApprove'
    })


    
    // await keypom.call(keypom, 'add_to_refund_allowlist', { account_id: minqi.accountId });
    
    let keypomBalance = await keypom.balance();
    console.log('keypom available INITIAL: ', keypomBalance.available.toString())
    console.log('keypom staked INITIAL: ', keypomBalance.staked.toString())
    console.log('keypom stateStaked INITIAL: ', keypomBalance.stateStaked.toString())
    console.log('keypom total INITIAL: ', keypomBalance.total.toString())

    // Save state for test runs
    t.context.worker = worker;
    t.context.accounts = { root, keypom, dao, daoBot, minqi, member1, member2, member3 };

    console.log("\u001b[1;35mDONE BEFOREEACH")
});

// If the environment is reused, use test.after to replace test.afterEach
test.afterEach(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});
test('Ensure DAO Bot Membership', async t => {
    const { keypom, dao, daoBot, minqi, member1, member2, member3 } = t.context.accounts;
    let daoBotMembership: Array<String> = await minqi.call(daoBot, 'view_user_roles', {dao_contract: dao.accountId, member: daoBot.accountId});
    t.is(daoBotMembership.includes('keypom-daobot'), true);
});

test('Normal Claiming Process', async t => {
    const { keypom, dao, daoBot, minqi, member1, member2, member3 } = t.context.accounts;

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
                    attached_deposit: NEAR.parse("1.5").toString()
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
    
    let member1_groups: Array<String> = await minqi.call(daoBot, 'view_user_roles', {dao_contract: dao.accountId, member: member1.accountId});
    t.is(member1_groups.includes('new-onboardee-role'), true);
});
