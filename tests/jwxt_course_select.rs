mod common;

use njupt::jwxt::{SelectableSearch, Term};

#[tokio::test]
#[ignore = "needs NJUPT_USERNAME / NJUPT_PASSWORD and network"]
async fn search_selectable_courses_ok() {
    let jwxt = common::login().await;
    let ctx = jwxt.selection_context().await.expect("selection_context");
    assert!(!ctx.tabs.is_empty());
    assert!(
        ctx.tabs.iter().all(|t| !t.xkkz_xh.is_empty()),
        "tabs must carry xkkz_xh"
    );

    let year: u32 = ctx
        .field("xkxnm")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2026);
    let term = match ctx.field("xkxqm") {
        Some("12") => Term::Second,
        _ => Term::First,
    };

    let kklxdm = ctx
        .tabs
        .iter()
        .find(|t| t.kklxdm == "10")
        .map(|t| t.kklxdm.clone())
        .unwrap_or_else(|| ctx.tabs[0].kklxdm.clone());

    let result = jwxt
        .search_selectable_courses(
            &ctx,
            &SelectableSearch {
                year,
                term,
                kklxdm,
                filter: None,
                page_start: 1,
                page_end: 20,
                only_available: false,
            },
        )
        .await
        .expect("search_selectable_courses");

    assert_ne!(result.get("flag").and_then(|f| f.as_str()), Some("0"));
    let n = result
        .get("tmpList")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    eprintln!("selectable tmpList={n}, from_year={year}");
    assert!(result.get("tmpList").is_some());
}
