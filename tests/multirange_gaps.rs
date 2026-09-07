use villagesql::{InValue, VdfReturn};
use vsql_ranger_arranger::engine::Range;
use vsql_ranger_arranger::multirange_types::{
    datemr_encode, int4mr_encode, int8mr_encode, mr_decode_to_vec,
    roundtrip_components,
};
use vsql_ranger_arranger::subtype::{
    date::DateOps, datetime::DateTimeOps, int4::Int4Ops, int8::Int8Ops,
};

mod slice1 {
    use super::*;
    use vsql_ranger_arranger::func::setops::{
        datemr_contains_element, datemr_union, dtmr_contains_element, dtmr_union,
        int4mr_contains_element, int4mr_union, int8mr_contains_element, int8mr_union,
    };

    fn custom(buf: &[u8]) -> InValue<'_> {
        InValue::Custom(buf)
    }

    #[test]
    fn multirange_union_merges_components() {
        let a = int8mr_encode("{[1,2),[5,6)}").unwrap();
        let b = int8mr_encode("{[3,4)}").unwrap();
        let merged = match int8mr_union(&[custom(&a), custom(&b)]) {
            VdfReturn::Binary(bytes) => bytes,
            other => panic!("expected Binary, got {:?}", other),
        };
        let comps = mr_decode_to_vec::<Int8Ops>(&merged).unwrap();
        assert_eq!(comps.len(), 3);
        assert!(!comps[0].empty);
    }

    #[test]
    fn multirange_contains_element_true_when_contained() {
        let mr = int8mr_encode("{[1,5)}").unwrap();
        let range_buf = vsql_ranger_arranger::engine::canonical::range_to_bytes::<Int8Ops>(
            &Range {
                empty: false,
                lower_inf: false,
                upper_inf: false,
                lower_inc: true,
                upper_inc: false,
                lower: 3,
                upper: 4,
            },
        );
        assert!(matches!(
            int8mr_contains_element(&[custom(&range_buf), custom(&mr)]),
            VdfReturn::Int(1)
        ));
    }

    #[test]
    fn multirange_contains_element_false_when_not_contained() {
        let mr = int8mr_encode("{[1,2),[5,6)}").unwrap();
        let range_buf = vsql_ranger_arranger::engine::canonical::range_to_bytes::<Int8Ops>(
            &Range {
                empty: false,
                lower_inf: false,
                upper_inf: false,
                lower_inc: true,
                upper_inc: false,
                lower: 3,
                upper: 4,
            },
        );
        assert!(matches!(
            int8mr_contains_element(&[custom(&range_buf), custom(&mr)]),
            VdfReturn::Int(0)
        ));
    }

    #[test]
    fn multirange_contains_element_null_propagates() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_contains_element,
            int4mr_contains_element,
            datemr_contains_element,
            dtmr_contains_element,
        ];
        for f in cases {
            assert!(matches!(f(&[InValue::Null, custom(&[])]), VdfReturn::Null));
            assert!(matches!(f(&[custom(&[]), InValue::Null]), VdfReturn::Null));
        }
    }

    #[test]
    fn multirange_union_null_propagates() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_union,
            int4mr_union,
            datemr_union,
            dtmr_union,
        ];
        for f in cases {
            assert!(matches!(f(&[InValue::Null, custom(&[])]), VdfReturn::Null));
            assert!(matches!(f(&[custom(&[]), InValue::Null]), VdfReturn::Null));
        }
    }
}

mod setops_gaps {
    use super::*;
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

#[test]
fn multirange_helpers_error_paths() {
    assert!(int8mr_encode("[5,1)").is_err());
    assert!(int4mr_encode("[5,1)").is_err());
    assert!(datemr_encode("[2026-01-10,2026-01-01)").is_err());
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

mod slice2 {
    use super::*;
    use vsql_ranger_arranger::func::extract::{
        datemr_isempty, datemr_length, datemr_upper,
        dtmr_isempty, dtmr_length, dtmr_upper,
        int4mr_isempty, int4mr_length, int4mr_upper,
        int8mr_isempty, int8mr_length, int8mr_upper,
    };

    fn custom(buf: &[u8]) -> InValue<'_> {
        InValue::Custom(buf)
    }

    #[test]
    fn upper_returns_canonicalized_literal() {
        let mr = int8mr_encode("{[1,3),[7,10]}").unwrap();
        assert!(matches!(int8mr_upper(&[custom(&mr)]), VdfReturn::String(s) if !s.is_empty()));
    }

    #[test]
    fn isempty_false_when_non_empty() {
        let mr = int8mr_encode("{[1,3)}").unwrap();
        assert!(matches!(int8mr_isempty(&[custom(&mr)]), VdfReturn::Int(0)));
    }

    #[test]
    fn isempty_true_when_empty() {
        let mr = int8mr_encode("empty").unwrap();
        assert!(matches!(int8mr_isempty(&[custom(&mr)]), VdfReturn::Int(1)));
    }

    #[test]
    fn length_equals_component_count() {
        let adjacent = "{[1,5)}";
        let mr = int8mr_encode(adjacent).unwrap();
        assert!(matches!(int8mr_length(&[custom(&mr)]), VdfReturn::Int(1)));
        let gap = "{[1,2),[5,6)}";
        let mr2 = int8mr_encode(gap).unwrap();
        assert!(matches!(int8mr_length(&[custom(&mr2)]), VdfReturn::Int(2)));
    }

    #[test]
    fn slice2_null_propagates() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_upper,
            int8mr_isempty,
            int8mr_length,
            int4mr_upper,
            int4mr_isempty,
            int4mr_length,
            datemr_upper,
            datemr_isempty,
            datemr_length,
            dtmr_upper,
            dtmr_isempty,
            dtmr_length,
        ];
        for f in cases {
            assert!(matches!(f(&[InValue::Null]), VdfReturn::Null));
        }
    }

    #[test]
    fn slice2_invalid_arg_count_errors() {
        let cases: &[fn(&[InValue<'_>]) -> VdfReturn] = &[
            int8mr_upper,
            int8mr_isempty,
            int8mr_length,
        ];
        for f in cases {
            assert!(matches!(f(&[]), VdfReturn::Error(_)));
        }
    }
}
