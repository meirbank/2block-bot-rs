use ethers::{prelude::*};
use ethers::core::{rand::thread_rng, types::transaction::eip2718::TypedTransaction};
use ethers::types::{Address,  H256, I256, U128, U256};
use ethers::providers::{Provider};
use ethers_providers::Ws;
use rand::prelude::*;
use ethers_flashbots::*;
use url::Url;
use std::time::Duration;
use std::cmp;
use num::pow::pow;
use std::process;
use dotenv;
use bytes::Bytes;
use std::{convert::TryFrom, sync::Arc};


#[tokio::main]
async fn main() -> eyre::Result<()> {

    /*async fn mint_with_flashbots() -> eyre::Result<()> {

        let flashbots_provider = Provider::<Http>::try_from("https://mainnet.eth.aragon.network").unwrap();
        let pk = dotenv::var("PK").unwrap();
        let signer_wallet = pk.parse::<LocalWallet>()?;
        let bundle_wallet = pk.parse::<LocalWallet>()?;
    
        let client = SignerMiddleware::new(
            FlashbotsMiddleware::new(
                flashbots_provider,
                Url::parse("https://relay.flashbots.net")?,
                bundle_wallet,
            ),
            signer_wallet,
        );

        // get last block number
        let block_number = client.get_block_number().await?;

        let recipient = "0x0e824d5f934Ad01A603101d6A41D723C1702822B".parse::<Address>().unwrap();

        // Build a custom bundle that pays 0x0000000000000000000000000000000000000000
        let tx = {
            let mut inner: TypedTransaction = TransactionRequest::new().to(recipient).value(0).data(ethers::core::types::Bytes::from(bytes::Bytes::from("0x1249c58b"))).gas_price(100).gas(200000).into();
            client.fill_transaction(&mut inner, None).await?;
            inner
        };
        let signature = client.signer().sign_transaction(&tx).await?;
        let bundle = BundleRequest::new()
            .push_transaction(tx.rlp_signed(42, &signature))
            .set_block(block_number + 1)
            .set_simulation_block(block_number)
            .set_simulation_timestamp(0);

        // Simulate it
        //let simulated_bundle = client.inner().simulate_bundle(&bundle).await?;
        //println!("Simulated bundle: {:?}", simulated_bundle);

        // Send it
        let pending_bundle = client.inner().send_bundle(&bundle).await?;

        println!("Bundle sent");

        // You can also optionally wait to see if the bundle was included
        match pending_bundle.await {
            Ok(bundle_hash) => println!(
                "Bundle with hash {:?} was included in target block",
                bundle_hash
            ),
            Err(PendingBundleError::BundleNotIncluded) => {
                println!("Bundle was not included in target block.")
            }
            Err(e) => println!("An error occured: {}", e),
        }

        Ok(())

    }
    */
    

    let pk = dotenv::var("PK").unwrap();

    let ethers_provider = Arc::new({
        // switch to websocket provider later?
        let provider = Provider::<Http>::try_from("https://mainnet.eth.aragon.network").unwrap();

        let chain_id = provider.get_chainid().await?;

        // this wallet's private key
        let wallet = pk.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());

        SignerMiddleware::new(provider, wallet)
    });


    let ws = Ws::connect("wss://mainnet.infura.io/ws/v3/4af36def3746417b9b93290790e33f23").await?;
    let signer_wallet = pk.parse::<LocalWallet>()?;
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?.take(100);
    let mut diffculties: Vec<i128> = Vec::new();
    let mut has_sent_anything = false;
    let ttd: i128 = 58750000000000000000000;
    // const estimatedNumberOfBlocksUntilMerge = ttd.minus(totalDifficulty).dividedBy(avgBlockDifficulty).decimalPlaces(0);
    fn get_average(vec: &Vec<i128>) -> i128{
        let totalsum = vec.iter().sum::<i128>() as i128;
        totalsum / vec.len() as i128
    }
    while let Some(block) = stream.next().await {
        let block = provider.get_block(block).await?.unwrap();
        let current_difficulty = block.difficulty.as_u128() as i128;
        let total_difficulty = block.total_difficulty.unwrap().as_u128() as i128;
        diffculties.push(current_difficulty);
        let blocksUntilMerge = (ttd - total_difficulty) / get_average(&diffculties);
        println!("ttd {}, total {}, current {}, average {}", ttd, total_difficulty, current_difficulty, get_average(&diffculties) );
        println!("Estimated blocks to merge {}", blocksUntilMerge);
        if total_difficulty < ttd && blocksUntilMerge == 1 && !has_sent_anything {
            has_sent_anything = true;
            println!("LAST BLOCK");
            // mint_with_flashbots();
        }
    }

    Ok(())
}