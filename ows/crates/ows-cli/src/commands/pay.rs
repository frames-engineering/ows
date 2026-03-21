use crate::commands::read_passphrase;
use crate::CliError;

/// `ows pay request <url> --wallet <name> [--method GET] [--body '{}']`
pub fn run(
    url: &str,
    wallet_name: &str,
    method: &str,
    body: Option<&str>,
    skip_passphrase: bool,
) -> Result<(), CliError> {
    let passphrase = if skip_passphrase {
        String::new()
    } else {
        read_passphrase().to_string()
    };

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| CliError::InvalidArgs(format!("tokio: {e}")))?;

    let result = rt.block_on(ows_pay::x402::pay(
        wallet_name,
        &passphrase,
        url,
        method,
        body,
    ))?;

    if !result.amount_display.is_empty() {
        eprintln!(
            "Paid {} on {} via x402",
            result.amount_display, result.network
        );
    }

    if result.status >= 400 {
        eprintln!("HTTP {}", result.status);
    }

    println!("{}", result.body);
    Ok(())
}

/// `ows pay discover [--query <search>]`
pub fn discover(query: Option<&str>) -> Result<(), CliError> {
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| CliError::InvalidArgs(format!("tokio: {e}")))?;

    // Fetch both x402 and MPP in parallel.
    let (x402_services, mpp_services) = rt.block_on(async {
        let x402 = async {
            match query {
                Some(q) => ows_pay::discovery::search(q).await,
                None => ows_pay::discovery::discover_x402(Some(100), None).await,
            }
        };
        let mpp = async {
            match query {
                Some(q) => ows_pay::discovery::search_mpp(q).await,
                None => ows_pay::discovery::discover_mpp().await,
            }
        };
        tokio::join!(x402, mpp)
    });

    // --- MPP services ---
    let mpp = mpp_services.unwrap_or_default();
    if !mpp.is_empty() {
        eprintln!("MPP ({} services):\n", mpp.len());
        for svc in &mpp {
            // Find cheapest paid endpoint for price display.
            let price = cheapest_mpp_price(svc);

            let cats = svc.categories.join(", ");
            let desc = if svc.description.len() > 80 {
                format!("{}...", &svc.description[..77])
            } else {
                svc.description.clone()
            };

            let paid_count = svc.endpoints.iter().filter(|e| e.payment.is_some()).count();
            let total_count = svc.endpoints.len();

            println!(
                "  {:<25} {:>8}  {paid_count:>2}/{total_count:<2} endpoints  [{cats}]",
                svc.name, price
            );
            println!("  {:25} {desc}", "");
            println!("  {:25} {}", "", svc.service_url);
            println!();
        }
    }

    // --- x402 services (mainnet only) ---
    let x402 = x402_services.unwrap_or_default();
    const TESTNETS: &[&str] = &[
        "base-sepolia",
        "eip155:84532",
        "eip155:11155111",
        "solana-devnet",
    ];
    let filtered: Vec<_> = x402
        .iter()
        .filter(|svc| {
            let net = svc
                .accepts
                .first()
                .map(|a| a.network.as_str())
                .unwrap_or("");
            !TESTNETS.iter().any(|t| net.contains(t))
        })
        .collect();

    if !filtered.is_empty() {
        eprintln!("x402 ({} services):\n", filtered.len());
        for svc in &filtered {
            let accept = svc.accepts.first();

            let price = accept
                .map(|a| ows_pay::discovery::format_usdc(&a.amount))
                .unwrap_or_else(|| "?".into());

            let network = accept.map(|a| a.network.as_str()).unwrap_or("?");

            let desc = accept
                .and_then(|a| a.description.as_deref())
                .or_else(|| svc.metadata.as_ref().and_then(|m| m.description.as_deref()))
                .unwrap_or("");

            let first_line = desc.lines().next().unwrap_or("");
            let desc_display = if first_line.len() > 80 {
                format!("{}...", &first_line[..77])
            } else {
                first_line.to_string()
            };

            println!("  {price:>8}  {network:<8}  {desc_display}");
            println!("  {:>8}  {:8}  {}", "", "", svc.resource);
            println!();
        }
    }

    if mpp.is_empty() && filtered.is_empty() {
        eprintln!("No services found.");
    }

    Ok(())
}

fn cheapest_mpp_price(svc: &ows_pay::types::MppService) -> String {
    svc.endpoints
        .iter()
        .filter_map(|e| e.payment.as_ref())
        .filter_map(|p| {
            let amt_str = p.amount.as_deref()?;
            let decimals = p.decimals.unwrap_or(6);
            let amt: u128 = amt_str.parse().ok()?;
            Some((amt, decimals))
        })
        .min_by_key(|(amt, _)| *amt)
        .map(|(amt, dec)| ows_pay::discovery::format_mpp_amount(&amt.to_string(), dec))
        .unwrap_or_else(|| "free".into())
}
