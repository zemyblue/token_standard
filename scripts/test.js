#!/usr/bin/env -S yarn node

// This script should be executed after optimize a smart contract to artifacts directory (`cargo otpimize`)

const { SigningFinschiaClient, makeLinkPath } = require("@finschia/finschia");
const { DirectSecp256k1HdWallet } = require("@cosmjs/proto-signing");
const { calculateFee, GasPrice } = require("@cosmjs/stargate");
const fs = require("fs");

const endpoint = "http://localhost:26657";
const alice = {
    mnemonic: "mind flame tobacco sense move hammer drift crime ring globe art gaze cinnamon helmet cruise special produce notable negative wait path scrap recall have",
    address0: "link146asaycmtydq45kxc8evntqfgepagygelel00h",
    address1: "link1aaffxdz4dwcnjzumjm7h89yjw5c5wul88zvzuu",
    address2: "link1ey0w0xj9v48vk82ht6mhqdlh9wqkx8enkpjwpr",
    address3: "link1dfyywjglcfptn72axxhsslpy8ep6wq7wujasma",
}

// instantiate
const inits = [
    {
        label: "From deploy test.js",
        msg: {
            name: "A To Z Token",
            symbol: "ATZ",
            decimals: 6,
            initial_balances: "1000000000",
        },
        admin: alice.address0,
    },
    {
        label: "From deploy(2) test.js",
        msg: {
            name: "Second Token",
            symbol: "SCD",
            decimals: 6,
            initial_balances: "1000000000",
        },
        admin: alice.address1,
    },
];

class TokenClient {
    constructor(client, codeId) {
        this.client = client;
        this.codeId = codeId;
        this.gasPrice = GasPrice.fromString("0.025cony");
    }

    async instantiateContract(sender, initData) {
        const instantiateFee = calculateFee(500_000, this.gasPrice);
        const instRes = await this.client.instantiate(
            sender, 
            this.codeId, 
            initData.msg, 
            initData.label, 
            instantiateFee,
             {
                memo: "Create a Test Token",
                admin: initData.admin,
            }
        );
        console.info(`Contract instantiated at ${instRes.contractAddress} in ${instRes.transactionHash}`);
    
        return instRes.contractAddress;
    }

    async transfer(contractAddress, sender, recipient, amount, gasLimit = 150_000) {
        const transferFee = calculateFee(gasLimit, this.gasPrice);
        const transferMsg = {
            transfer: {
                recipient: recipient,
                amount: amount,
            },
        };
        
        try {
            const tranRes = await this.client.execute(sender, contractAddress, transferMsg, transferFee);
            console.info(`Transfer txHash: ${tranRes.transactionHash}, events: ${JSON.stringify(tranRes.events)}`);

            return tranRes.transactionHash;    
        } catch (error) {
            return error;
        }
    }

    async approve(contractAddress, sender, spender, amount, current_allowance) {
        const approveFee = calculateFee(150_000, this.gasPrice);
        const approveMsg = {
            approve: {
                spender: spender,
                amount: amount,
                current_allowance: current_allowance,
            },
        };
        const approveRes = await this.client.execute(sender, contractAddress, approveMsg, approveFee);
        console.info(`Approve txHash: ${approveRes.transactionHash}, events: ${JSON.stringify(approveRes.events)}`);

        return approveRes.transactionHash;
    }

    async transferFrom(contractAddress, sender, owner, recipient, amount) {
        const transferFromFee = calculateFee(150_000, this.gasPrice);
        const transferFromMsg = {
            transfer_from: {
                owner: owner,
                recipient: recipient,
                amount: amount,
            },
        };
        const tran2Res = await this.client.execute(sender, contractAddress, transferFromMsg, transferFromFee);
        console.info(`TransferFrom txHash: ${tran2Res.transactionHash}, events: ${JSON.stringify(tran2Res.events)}`);

        return tran2Res.transactionHash;
    }

    async balance(contractAddress, owner) {
        const query = {balance: { owner: owner }};
        const res = await this.client.queryContractSmart(contractAddress, query);
        return res.balance;
    }
}


class CallerClient {
    constructor(client, codeId) {
        this.client = client;
        this.codeId = codeId;
        this.gasPrice = GasPrice.fromString("0.025cony");
    }

    async instantiateContract(sender) {
        const instantiateFee = calculateFee(500_000, this.gasPrice);
        const instRes = await this.client.instantiate(
            sender, 
            this.codeId, 
            {}, 
            "caller instantiate", 
            instantiateFee,
             {
                memo: "Create a Test Token",
                admin: sender,
            }
        );
        console.info(`Contract instantiated at ${instRes.contractAddress} in ${instRes.transactionHash}`);
    
        return instRes.contractAddress;
    }

    async transfer(contractAddress, sender, contract, recipient, amount) {
        const fee = calculateFee(200_000, this.gasPrice);
        const msg = {
            transfer: {
                contract: contract,
                recipient: recipient,
                amount: amount,
            },
        }
        const res = await this.client.execute(sender, contractAddress, msg, fee);
        console.info(`Transfer(caller) txHash: ${res.transactionHash}, events: ${JSON.stringify(res.events)}`);

        return res.transactionHash;
    }

    async transferFrom(contractAddress, sender, contract, owner, recipient, amount) {
        const fee = calculateFee(200_000, this.gasPrice);
        const msg = {
            transfer_from: {
                contract: contract,
                owner: owner,
                recipient: recipient,
                amount: amount,
            },
        };
        const res = await this.client.execute(sender, contractAddress, msg, fee);
        console.info(`TransferFrom(caller) txHash: ${res.transactionHash}, events: ${JSON.stringify(res.events)}`);

        return res.transactionHash;
    }

    async approve(contractAddress, sender, contract, spender, amount, current_allowance) {
        const approveFee = calculateFee(200_000, this.gasPrice);
        const approveMsg = {
            approve: {
                contract: contract,
                spender: spender,
                amount: amount,
                current_allowance: current_allowance,
            },
        };
        const res = await this.client.execute(sender, contractAddress, approveMsg, approveFee);
        console.info(`Approve(caller) txHash: ${res.transactionHash}, events:${JSON.stringify(res.events)}`);

        return res.transactionHash;
    }
}

async function deployContract(client, sender, wasmData) {
    const gasPrice = GasPrice.fromString("0.025cony");
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(sender, wasmData, uploadFee, "Upload standard contract");
    console.info(`Upload succeeded. Receipt: ${JSON.stringify(uploadReceipt)}`);

    return uploadReceipt.codeId;
}


async function main() {
    console.info("test");
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(alice.mnemonic, {
        hdPaths: [makeLinkPath(0), makeLinkPath(1), makeLinkPath(2), makeLinkPath(3)],
        prefix: "link",
    });
    const client = await SigningFinschiaClient.connectWithSigner(endpoint, wallet);

    // deploy
    const wasm = fs.readFileSync(__dirname + "/../artifacts/token_standard.wasm");
    const tokenCodeId = await deployContract(client, alice.address0, wasm);
    
    const tokenClient = new TokenClient(client, tokenCodeId);

    // instantiate 1
    const contractAddress1 = await tokenClient.instantiateContract(alice.address0, inits[0]);

    // transfer
    await tokenClient.transfer(contractAddress1, alice.address0, alice.address1, "1000");

    // approve
    await tokenClient.approve(contractAddress1, alice.address0, alice.address2, "1000000", "0");

    // transferFrom
    await tokenClient.transferFrom(contractAddress1, alice.address2, alice.address0, alice.address3, "100000");

    // instantiate 2
    const contractAddress2 = await tokenClient.instantiateContract(alice.address1, inits[1]);

    // transfer to contractAddress2
    console.info("[Transfer token to other contract]");
    await tokenClient.transfer(contractAddress1, alice.address0, contractAddress2, "1000", 200_000);

    // transfer to native module (x/foundation)
    // grpcurl -plaintext -d '{"name":"foundation"}' 127.0.0.1:9090 cosmos.auth.v1beta1.Query/ModuleAccountByName
    // address: link190vt0vxc8c8vj24a7mm3fjsenfu8f5yxxj76cp
    const foundationAddress = "link190vt0vxc8c8vj24a7mm3fjsenfu8f5yxxj76cp";
    await tokenClient.transfer(contractAddress1, alice.address0, foundationAddress, "10000");

    /////////////////////////////////////////
    // deploy caller contract
    const callerWasm = fs.readFileSync(__dirname + "/../artifacts/token_caller.wasm");
    const callerCodeId = await deployContract(client, alice.address1, callerWasm);

    const callerClient = new CallerClient(client, callerCodeId);

    // instantiate
    const callerContractAddr = await callerClient.instantiateContract(alice.address1);

    // trasnfer token to caller contract
    console.info("[Transfer token to callerContract]");
    const bal1 = await tokenClient.balance(contractAddress1, callerContractAddr);
    console.info(`\ncontract balance1: ${bal1}`);
    const ret = await tokenClient.transfer(contractAddress1, alice.address0, callerContractAddr, "10000", 200_000);
    const bal2 = await tokenClient.balance(contractAddress1, callerContractAddr);
    console.info(`\ncontract balance2: ${bal2}`);

    // transfer by caller contract
    console.info("[Transfer token from callerContract to alice address2]");
    await callerClient.transfer(callerContractAddr, alice.address1, contractAddress1, alice.address2, "5000");

    // approve by caller contract
    const callerContractAddr2 = await callerClient.instantiateContract(alice.address2);
    console.info("[Approve token of caller]");
    await callerClient.approve(callerContractAddr, alice.address1, contractAddress1, callerContractAddr2, "5000", "0");

    console.info("[TransferFrom by caller]");
    await callerClient.transferFrom(callerContractAddr2, alice.address2, contractAddress1, callerContractAddr, alice.address3, "2000");
}


main().then(
    () => {
        console.info("All done");
        process.exit(0);
    },
    (error) => {
        console.error(error);
        process.exit(1);
    }
)
