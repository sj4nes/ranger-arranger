use villagesql::VdfReturn;
use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::multirange_types::{
    datemr_encode, int4mr_encode, int8mr_decode, int8mr_encode, mr_decode_to_vec,
    roundtrip_components,
};
use vsql_ranger_arranger::subtype::{
    date::DateOps, datetime::DateTimeOps, int4::Int4Ops, int8::Int8Ops,
};

// Null/error branch coverage for multirange setops entry points.
mod setops_gaps {
    use villagesql::InValue;
    use villagesql::VdfReturn;
    use vsql_ranger_arranger::func::setops::{
        datemr_contains_range, datemr_difference, datemr_intersect, datemr_merge, datemr_overlaps,
        dtmr_contains_range, dtmr_difference, dtmr_intersect, dtmr_merge, dtmr_overlaps,
        int4mr_contains_range, int4mr_difference, int4mr_intersect, int4mr_merge, int4mr_overlaps,
        int8mr_contains_range, int8mr_difference, int8mr_intersect, int8mr_merge, int8mr_overlaps,
    };

    fn custom(buf: &[u8]) -> InValue<'_> {
        InValue::Custom(buf)
    }

    #[test]
    fn null_first_arg_propagates() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_overlaps,
            int8mr_contains_range,
            int8mr_intersect,
            int8mr_merge,
            int8mr_difference,
            int4mr_overlaps,
            int4mr_contains_range,
            int4mr_intersect,
            int4mr_merge,
            int4mr_difference,
            datemr_overlaps,
            datemr_contains_range,
            datemr_intersect,
            datemr_merge,
            datemr_difference,
            dtmr_overlaps,
            dtmr_contains_range,
            dtmr_intersect,
            dtmr_merge,
            dtmr_difference,
        ];
        for f in cases {
            assert!(matches!(f(&[InValue::Null, custom(&[])]), VdfReturn::Null));
        }
    }

    #[test]
    fn null_second_arg_propagates() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_overlaps,
            int8mr_contains_range,
            int8mr_intersect,
            int8mr_merge,
            int8mr_difference,
        ];
        for f in cases {
            assert!(matches!(f(&[custom(&[]), InValue::Null]), VdfReturn::Null));
        }
    }

    #[test]
    fn wrong_arg_count_returns_error() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_overlaps,
            int8mr_intersect,
            int8mr_merge,
            int8mr_difference,
        ];
        for f in cases {
            assert!(matches!(f(&[custom(&[])]), VdfReturn::Error(_)));
        }
    }
}

// Error and edge-case coverage in multirange helpers.
#[test]
fn multirange_helpers_error_paths() {
    // Empty multirange literal -> encode/decoded bytes must be exact.
    let enc = int8mr_encode("empty").unwrap();
    assert_eq!(int8mr_decode(&enc).unwrap(), "{}");

    // Reject reversed bounds cleanly.
    assert!(int8mr_encode("[5,1)").is_err());
    assert!(int4mr_encode("[5,1)").is_err());
    assert!(datemr_encode("[2026-01-10,2026-01-01)").is_err());

    // Mismatched subtype decode returns error rather than panicking.
    assert!(mr_decode_to_vec::<Int8Ops>(&[]).is_err());
    assert!(mr_decode_to_vec::<Int4Ops>(&[]).is_err());
    assert!(mr_decode_to_vec::<DateOps>(&[]).is_err());
    assert!(mr_decode_to_vec::<DateTimeOps>(&[]).is_err());
}

#[test]
fn multirange_roundtrip_components_covers_subtypes() {
    let cases = &[
        (
            "{[1,5)}",
            roundtrip_components::<Int8Ops> as fn(&str) -> Result<Vec<Range>, String>,
        ),
        (
            "{[1,5)}",
            roundtrip_components::<Int4Ops> as fn(&str) -> Result<Vec<Range>, String>,
        ),
        (
            "{[2026-01-01,2026-01-05)}",
            roundtrip_components::<DateOps> as fn(&str) -> Result<Vec<Range>, String>,
        ),
        (
            "{[2026-01-01 00:00:00,2026-01-05 00:00:00)}",
            roundtrip_components::<DateTimeOps> as fn(&str) -> Result<Vec<Range>, String>,
        ),
    ];
    for (lit, f) in cases {
        let comps = f(lit).expect("valid multirange literal");
        assert!(!comps.is_empty());
    }
}

#[test]
fn multirange_lib_wrappers_expose_errors() {
    use villagesql::InValue;
    use vsql_ranger_arranger::func::setops::{
        datemr_contains_range, datemr_difference, datemr_intersect, datemr_merge, datemr_overlaps,
        dtmr_contains_range, dtmr_difference, dtmr_intersect, dtmr_merge, dtmr_overlaps,
        int4mr_contains_range, int4mr_difference, int4mr_intersect, int4mr_merge, int4mr_overlaps,
        int8mr_contains_range, int8mr_difference, int8mr_intersect, int8mr_merge, int8mr_overlaps,
    };

    let empty = &[InValue::Custom(&[])];
    assert!(matches!(int8mr_overlaps(empty), VdfReturn::Error(_)));
    assert!(matches!(int8mr_intersect(empty), VdfReturn::Error(_)));
    assert!(matches!(int8mr_merge(empty), VdfReturn::Error(_)));
    assert!(matches!(int8mr_difference(empty), VdfReturn::Error(_)));
    assert!(matches!(int8mr_contains_range(empty), VdfReturn::Error(_)));

    assert!(matches!(int4mr_overlaps(empty), VdfReturn::Error(_)));
    assert!(matches!(int4mr_intersect(empty), VdfReturn::Error(_)));
    assert!(matches!(int4mr_merge(empty), VdfReturn::Error(_)));
    assert!(matches!(int4mr_difference(empty), VdfReturn::Error(_)));
    assert!(matches!(int4mr_contains_range(empty), VdfReturn::Error(_)));

    assert!(matches!(datemr_overlaps(empty), VdfReturn::Error(_)));
    assert!(matches!(datemr_intersect(empty), VdfReturn::Error(_)));
    assert!(matches!(datemr_merge(empty), VdfReturn::Error(_)));
    assert!(matches!(datemr_difference(empty), VdfReturn::Error(_)));
    assert!(matches!(datemr_contains_range(empty), VdfReturn::Error(_)));

    assert!(matches!(dtmr_overlaps(empty), VdfReturn::Error(_)));
    assert!(matches!(dtmr_intersect(empty), VdfReturn::Error(_)));
    assert!(matches!(dtmr_merge(empty), VdfReturn::Error(_)));
    assert!(matches!(dtmr_difference(empty), VdfReturn::Error(_)));
    assert!(matches!(dtmr_contains_range(empty), VdfReturn::Error(_)));
}

#[test]
fn multirange_decode_to_vec_empty_input_errors() {
    assert!(mr_decode_to_vec::<Int8Ops>(&[]).is_err());
    assert!(mr_decode_to_vec::<Int4Ops>(&[]).is_err());
    assert!(mr_decode_to_vec::<DateOps>(&[]).is_err());
    assert!(mr_decode_to_vec::<DateTimeOps>(&[]).is_err());
}
