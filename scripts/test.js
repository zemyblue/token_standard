#!/usr/bin/env -S yarn node

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

async function main() {
    console.info("test");
    const gasPrice = GasPrice.fromString("0.025cony");
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(alice.mnemonic, {
        hdPaths: [makeLinkPath(0), makeLinkPath(1), makeLinkPath(2), makeLinkPath(3)],
        prefix: "link",
    });
    const client = await SigningFinschiaClient.connectWithSigner(endpoint, wallet);

    const wasm = fs.readFileSync(__dirname + "/../artifacts/token_standard.wasm");

    // deploy
    const uploadFee = calculateFee(1_500_000, gasPrice);
    const uploadReceipt = await client.upload(alice.address0, wasm, uploadFee, "Upload standard contract");
    console.info(`Upload succeeded. Receipt: ${JSON.stringify(uploadReceipt)}`);
    
    // instantiate
    const init = {
        label: "From deploy test.js",
        msg: {
            name: "A To Z Token",
            symbol: "ATZ",
            decimals: 6,
            initial_balances: "1000000000",
        },
        admin: alice.address0,
    };

    const instantiateFee = calculateFee(500_000, gasPrice);
    const instRes = await client.instantiate(
        alice.address0, 
        uploadReceipt.codeId, 
        init.msg, 
        init.label, 
        instantiateFee,
         {
            memo: "Create a Test Token",
            admin: init.admin,
        }
    );
    console.info(`Contract instantiated at ${instRes.contractAddress} in ${instRes.transactionHash}`);

    const contractAddress = instRes.contractAddress;
    // const contractAddress = "link1436kxs0w2es6xlqpp9rd35e3d0cjnw4sv8j3a7483sgks29jqwgstwt0ss";

    // transfer
    const transferFee = calculateFee(150_000, gasPrice);
    const transferMsg = {
        transfer: {
            recipient: alice.address1,
            amount: "1000",
        },
    };
    const tranRes = await client.execute(alice.address0, contractAddress, transferMsg, transferFee);
    console.info(`Transfer txHash: ${tranRes.transactionHash}, events: ${JSON.stringify(tranRes.events)}`);

    // approve
    const approveFee = calculateFee(150_000, gasPrice);
    const approveMsg = {
        approve: {
            spender: alice.address2,
            amount: "1000000",
            current_allowance: "0",
        },
    };
    const approveRes = await client.execute(alice.address0, contractAddress, approveMsg, approveFee);
    console.info(`Approve txHash: ${approveRes.transactionHash}, events: ${JSON.stringify(approveRes.events)}`);

    // transferFrom
    const transferFromFee = calculateFee(150_000, gasPrice);
    const transferFromMsg = {
        transfer_from: {
            owner: alice.address0,
            recipient: alice.address3,
            amount: "100000",
        },
    };
    const tran2Res = await client.execute(alice.address2, contractAddress, transferFromMsg, transferFromFee);
    console.info(`TransferFrom txHash: ${tran2Res.transactionHash}, events: ${JSON.stringify(tran2Res.events)}`);
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
