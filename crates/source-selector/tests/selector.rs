use std::sync::Arc;

use source_selector::{Config, MultiSelector, P2cConfig, RegisterError, Strategy, SwrrConfig};

fn selector(alpha: f32, seed: u64) -> MultiSelector<u8> {
    MultiSelector::new(
        Config {
            default: Strategy::P2c,
            ewma_alpha: alpha,
            p2c: P2cConfig { seed },
            swrr: SwrrConfig { temperature: 1.0 },
        },
        |v| *v,
    )
}

fn swrr_selector(temperature: f32) -> MultiSelector<u8> {
    MultiSelector::new(
        Config {
            default: Strategy::SoftmaxSwrr,
            ewma_alpha: 1.0,
            p2c: P2cConfig { seed: 1 },
            swrr: SwrrConfig { temperature },
        },
        |v| *v,
    )
}

#[test]
fn empty_pool_and_single_source() {
    let selector = selector(1.0, 1);

    assert!(selector.is_empty());
    assert_eq!(selector.select(), None);
    assert_eq!(selector.swrr_select(), None);

    selector.register("only").unwrap();
    assert_eq!(selector.len(), 1);
    assert_eq!(selector.select().as_deref(), Some("only"));
    assert_eq!(selector.swrr_select().as_deref(), Some("only"));
}

#[test]
fn duplicate_register_and_remove() {
    let selector = selector(1.0, 1);

    assert!(selector.register("a").is_ok());

    match selector.register("a") {
        Err(RegisterError::AlreadyRegistered { id }) => assert_eq!(id, "a"),
        other => panic!("unexpected: {other:?}"),
    }

    assert!(selector.remove("a"));
    assert!(!selector.remove("a"));
    assert!(selector.is_empty());
}

#[test]
fn prefers_lower_score() {
    let selector = selector(1.0, 1);

    selector.register("slow").unwrap();
    selector.register("fast").unwrap();
    assert!(selector.report("slow", 200));
    assert!(selector.report("fast", 10));

    for _ in 0..100 {
        assert_eq!(selector.p2_select().as_deref(), Some("fast"));
    }
}

#[test]
fn unreported_source_starts_at_zero() {
    let selector = selector(1.0, 1);

    selector.register("new").unwrap();
    selector.register("old").unwrap();
    assert!(selector.report("old", 1));

    for _ in 0..100 {
        assert_eq!(selector.p2_select().as_deref(), Some("new"));
    }
}

#[test]
fn ewma_smooths_scores() {
    let selector = selector(0.5, 1);

    selector.register("a").unwrap();
    selector.register("b").unwrap();

    assert!(selector.report("a", 10));
    assert!(selector.report("a", 20)); // a = 15
    assert!(selector.report("b", 16)); // b = 16

    for _ in 0..100 {
        assert_eq!(selector.p2_select().as_deref(), Some("a"));
    }
}

#[test]
fn alpha_one_uses_latest_score() {
    let selector = selector(1.0, 1);

    selector.register("a").unwrap();
    selector.register("b").unwrap();

    assert!(selector.report("a", 10));
    assert!(selector.report("a", 20)); // a = 20
    assert!(selector.report("b", 16)); // b = 16

    for _ in 0..100 {
        assert_eq!(selector.p2_select().as_deref(), Some("b"));
    }
}

#[test]
fn score_range_and_unknown_id() {
    let selector = selector(1.0, 1);

    selector.register("a").unwrap();

    assert!(selector.report("a", 0));
    assert!(selector.report("a", 255));
    assert!(!selector.report("missing", 1));
}

#[test]
fn removed_source_is_not_selected() {
    let selector = selector(1.0, 1);

    selector.register("a").unwrap();
    selector.register("b").unwrap();
    assert!(selector.report("a", 0));
    assert!(selector.report("b", 200));

    assert!(selector.remove("a"));

    for _ in 0..100 {
        assert_eq!(selector.p2_select().as_deref(), Some("b"));
    }

    assert!(!selector.report("a", 1));
}

#[test]
fn tie_breaks_both_ways() {
    let selector = selector(1.0, 42);

    selector.register("a").unwrap();
    selector.register("b").unwrap();
    assert!(selector.report("a", 1));
    assert!(selector.report("b", 1));

    let mut saw_a = false;
    let mut saw_b = false;

    for _ in 0..200 {
        match selector.p2_select().as_deref() {
            Some("a") => saw_a = true,
            Some("b") => saw_b = true,
            other => panic!("unexpected: {other:?}"),
        }
    }

    assert!(saw_a);
    assert!(saw_b);
}

#[test]
fn p2c_prefers_fast_source_among_many() {
    let selector = selector(1.0, 9);

    for id in ["fast", "mid", "slow"] {
        selector.register(id).unwrap();
    }

    assert!(selector.report("fast", 1));
    assert!(selector.report("mid", 50));
    assert!(selector.report("slow", 100));

    let mut fast = 0;
    for _ in 0..1000 {
        if selector.p2_select().as_deref() == Some("fast") {
            fast += 1;
        }
    }

    // P2C 只比较随机抽到的两个，不会像全局最小值那样 100% 选 fast。
    assert!(fast > 500);
    assert!(fast < 900);
}

#[test]
fn swrr_round_robins_when_weights_are_equal() {
    let selector = swrr_selector(1.0);

    for id in ["a", "b", "c"] {
        selector.register(id).unwrap();
    }

    let expected = ["a", "b", "c"];

    for i in 0..6 {
        assert_eq!(
            selector.swrr_select().as_deref(),
            Some(expected[i % expected.len()])
        );
    }
}

#[test]
fn swrr_prefers_lower_score() {
    let selector = swrr_selector(1.0);

    selector.register("fast").unwrap();
    selector.register("slow").unwrap();
    assert!(selector.report("fast", 0));
    assert!(selector.report("slow", 10));

    for _ in 0..100 {
        assert_eq!(selector.swrr_select().as_deref(), Some("fast"));
    }
}

#[test]
fn select_uses_configured_default() {
    let selector = swrr_selector(1.0);

    selector.register("a").unwrap();
    selector.register("b").unwrap();

    let expected = ["a", "b"];

    for i in 0..4 {
        assert_eq!(
            selector.select().as_deref(),
            Some(expected[i % expected.len()])
        );
    }
}

#[test]
fn select_with_explicit_strategy() {
    let selector = selector(1.0, 1);

    selector.register("a").unwrap();
    selector.register("b").unwrap();

    assert!(selector.select_with(Strategy::P2c).is_some());
    assert!(selector.select_with(Strategy::SoftmaxSwrr).is_some());
}

#[test]
fn same_seed_same_sequence() {
    fn make() -> MultiSelector<u8> {
        let selector = selector(1.0, 12345);
        for id in ["a", "b", "c"] {
            selector.register(id).unwrap();
        }
        selector
    }

    let first = make();
    let second = make();

    let first_seq: Vec<_> = (0..100).map(|_| first.p2_select().unwrap()).collect();
    let second_seq: Vec<_> = (0..100).map(|_| second.p2_select().unwrap()).collect();

    assert_eq!(first_seq, second_seq);
}

#[test]
fn different_t_per_selector() {
    let numbers = MultiSelector::<u32>::new(Config::default(), |v| u8::try_from(*v).unwrap_or(255));

    numbers.register("n").unwrap();
    assert!(numbers.report("n", 42));

    let texts =
        MultiSelector::<String>::new(Config::default(), |v| u8::try_from(v.len()).unwrap_or(255));

    texts.register("s").unwrap();
    assert!(texts.report("s", "hello".to_owned()));
}

#[test]
fn re_register_resets_score_and_empty_pool() {
    let selector = selector(1.0, 1);

    selector.register("a").unwrap();
    assert!(selector.report("a", 255));
    assert!(selector.remove("a"));

    selector.register("a").unwrap();
    selector.register("b").unwrap();
    assert!(selector.report("b", 128));

    // a 重新注册后未上报，0 分，应该胜出。
    assert_eq!(selector.p2_select().as_deref(), Some("a"));

    assert!(selector.remove("a"));
    assert!(selector.remove("b"));
    assert_eq!(selector.p2_select(), None);
    assert_eq!(selector.swrr_select(), None);
}

#[test]
fn selector_is_send_sync_for_non_send_t() {
    struct NotSend {
        _rc: std::rc::Rc<()>,
    }

    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<MultiSelector<NotSend>>();
}

#[test]
fn concurrent_use_does_not_panic() {
    let selector = Arc::new(selector(0.2, 7));

    selector.register("shared-a").unwrap();
    selector.register("shared-b").unwrap();

    let handles: Vec<_> = (0..4)
        .map(|thread| {
            let selector = Arc::clone(&selector);

            std::thread::spawn(move || {
                for i in 0..500 {
                    let id = format!("t{thread}-{i}");

                    let _ = selector.register(id.clone());

                    let strategy = if i % 2 == 0 {
                        Strategy::P2c
                    } else {
                        Strategy::SoftmaxSwrr
                    };

                    if let Some(chosen) = selector.select_with(strategy) {
                        let _ = selector.report(&chosen, u8::try_from(i % 100).unwrap_or(0));
                    }

                    let _ = selector.remove(&id);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(selector.len(), 2);
}
