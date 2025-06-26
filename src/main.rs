use risc0_zkvm::{
    guest::{env, sha::Impl},
    sha::Sha256,
};

fn read_u64_input() -> u64 {
    let mut input_bytes = [0u8; 8];
    env::read_slice(&mut input_bytes);
    u64::from_le_bytes(input_bytes)
}

fn main() {
    // Read public input (JSON)
    let total_supply = read_u64_input();  // First 8 bytes
    let timestamp = read_u64_input();  // next 8 bytes

    // Read private input
    // Currently we do not use the private input in this example,
    // but it is read to demonstrate the process.
    // It reads from the same api provided in the docs
    let mut private_hex_bytes = Vec::new();
    env::read_slice(&mut private_hex_bytes);
    let private_json_hex = String::from_utf8(private_hex_bytes).unwrap();

    // Ideally the following values would be fetched from a database or another source.
    // For this example, we will use hardcoded values.
    let reserve_types: [&str; 3] = ["USDC", "ETH", "BTC"];
    let reserve_amounts: [u64; 3] = [1000, 2000, 3000];
    let reserve_values_usd: [u64; 3] = [5000, 10000, 15000];

    // Validate length consistency
    assert_eq!(
        reserve_types.len(),
        reserve_amounts.len(),
        "reserve_types vs reserve_amounts mismatch"
    );
    assert_eq!(
        reserve_amounts.len(),
        reserve_values_usd.len(),
        "reserve_amounts vs reserve_values_usd mismatch"
    );

    // Calculate total reserve USD
    let total_reserves_usd: u64 = 10_000_000;

    // Collateralization ratio (basis points)
    let collateralization_ratio = if total_supply > 0 {
        (total_reserves_usd * 10_000) / total_supply
    } else {
        0
    };

    // Build reserve breakdown
    let mut reserve_breakdown = String::from("[");
    for (i, reserve_type) in reserve_types.iter().enumerate() {
        let pct = if total_reserves_usd > 0 {
            (reserve_values_usd[i] * 10_000) / total_reserves_usd
        } else {
            0
        };
        if i != 0 {
            reserve_breakdown.push(',');
        }
        reserve_breakdown.push_str(&format!(
            "{{\"reserve_type\":\"{}\",\"percentage\":{}}}",
            reserve_type, pct
        ));
    }
    reserve_breakdown.push(']');

    let digest =
        Impl::hash_bytes(&[private_json_hex.as_bytes()].concat());
    let hash_hex = hex::encode(digest.as_bytes());

    // Build final proof JSON
    let proof_json = format!(
        "{{\"total_reserves_usd\":{},\"total_supply\":{},\"collateralization_ratio\":{},\
          \"timestamp\":{},\"reserve_breakdown\":{},\"data_hash\":\"{}\"}}",
        total_reserves_usd,
        total_supply,
        collateralization_ratio,
        timestamp,
        reserve_breakdown,
        hash_hex
    );

    env::commit_slice(digest.as_bytes());
    env::commit_slice(proof_json.as_bytes());
}
