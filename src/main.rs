use std::collections::HashMap;
use rand::Rng;
use std::env;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "optionChain")]
    option_chain: OptionChain,
}

#[derive(Debug, Deserialize)]
struct OptionChain {
    result: Vec<ResultItem>,
}

#[derive(Debug, Deserialize)]
struct ResultItem {
    strikes: Vec<f64>,
    // Add other fields if needed
}

async fn fetch_data(symbol: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let qs: u32 = rand::thread_rng().gen_range(1..2);
    let url = format!("https://query{}.finance.yahoo.com/v7/finance/options/{}", qs, symbol);
    let body = reqwest::get(&url).await?.text().await?;
    let data: Response = serde_json::from_str(&body)?;
    Ok(data)
}

fn calculate_max_pain(option_chain: &HashMap<i64, (f64, f64)>) -> f64 {
    let mut max_pain_strike = 0.0;
    let mut max_pain_value = f64::MAX;

    for (strike_price, (_call_open_interest, _put_open_interest)) in option_chain {
        let mut total_payout = 0.0;
        let strike_price = *strike_price as f64;

        for (s_price, (c_open_interest, p_open_interest)) in option_chain {
            let s_price = *s_price as f64;
            if s_price < strike_price {
                total_payout += s_price * c_open_interest;
            } else {
                total_payout += strike_price * c_open_interest;
            }

            if s_price > strike_price {
                total_payout += (s_price - strike_price) * p_open_interest;
            }
        }

        if total_payout < max_pain_value {
            max_pain_value = total_payout;
            max_pain_strike = strike_price;
        }
    }

    max_pain_strike
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a stock symbol as a command line argument.");
        return Ok(());
    }
    let symbol = &args[1];
    let response = fetch_data(symbol).await?;
    println!("{:?}", response);

    // Create the option chain
    let option_chain: HashMap<i64, (f64, f64)> = response.option_chain.result[0].strikes.iter().map(|&strike| (strike as i64, (0.0, 0.0))).collect();

    let max_pain_strike = calculate_max_pain(&option_chain);
    println!("The max pain strike price is: {}", max_pain_strike);

    Ok(())
}
