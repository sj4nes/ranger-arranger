/// Unit-style checks for the multirange accessors/predicates.
mod slice2 {
    use villagesql::{InValue, VdfReturn};
    use vsql_ranger_arranger::func::extract::{
        datemr_length, datemr_upper, dtmr_length, dtmr_upper, int4mr_length, int4mr_upper,
        int8mr_isempty, int8mr_length, int8mr_upper,
    };
    use vsql_ranger_arranger::multirange_types::{
        datemr_encode, dtmr_encode, int4mr_encode, int8mr_encode,
    };

    #[test]
    fn int8mr_upper_returns_last_component_literal() {
        let bytes = int8mr_encode("{[1,3),[7,10)}").unwrap();
        assert!(matches!(
            int8mr_upper(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::String(s) if s == "10"
        ));
    }

    #[test]
    fn int8mr_isempty_true_for_empty() {
        let bytes = int8mr_encode("{}").unwrap();
        assert!(matches!(
            int8mr_isempty(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Int(1)
        ));
    }

    #[test]
    fn int8mr_length_counts_components() {
        let bytes = int8mr_encode("{[1,3),[7,10)}").unwrap();
        assert!(matches!(
            int8mr_length(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Int(2)
        ));
    }

    #[test]
    fn empty_multirange_returns_null_upper() {
        let bytes = int8mr_encode("{}").unwrap();
        assert!(matches!(
            int8mr_upper(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Null
        ));
    }

    #[test]
    fn int4mr_surface_matches_int8mr() {
        let bytes = int4mr_encode("{[1,3),[7,10)}").unwrap();
        assert!(matches!(
            int4mr_upper(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::String(s) if s == "10"
        ));
        assert!(matches!(
            int4mr_length(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Int(2)
        ));
    }

    #[test]
    fn datemr_surface_matches_int8mr() {
        let bytes = datemr_encode("{[2026-01-01,2026-01-03),[2026-01-07,2026-01-10)}").unwrap();
        assert!(matches!(
            datemr_upper(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::String(s) if s == "2026-01-10"
        ));
        assert!(matches!(
            datemr_length(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Int(2)
        ));
    }

    #[test]
    fn dtmr_surface_matches_int8mr() {
        let bytes = dtmr_encode(
            "{[2026-01-01 00:00:00,2026-01-03 00:00:00),[2026-01-07 00:00:00,2026-01-10 00:00:00)}",
        )
        .unwrap();
        let out = dtmr_upper(&[InValue::Custom(bytes.as_slice())]);
        eprintln!("DEBUG dtmr_upper={:?}", out);
        assert!(matches!(
            dtmr_upper(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::String(s) if s == "2026-01-10 00:00:00.000000"
        ));
        assert!(matches!(
            dtmr_length(&[InValue::Custom(bytes.as_slice())]),
            VdfReturn::Int(2)
        ));
    }
}

/// Slice 3: multirange element accessor `NTH`.
mod slice3 {
    use villagesql::{InValue, VdfReturn};
    use vsql_ranger_arranger::func::extract::{datemr_nth, dtmr_nth, int4mr_nth, int8mr_nth};
    use vsql_ranger_arranger::multirange_types::{
        datemr_encode, dtmr_encode, int4mr_encode, int8mr_encode,
    };

    #[test]
    fn int8mr_nth_returns_first_component() {
        let bytes = int8mr_encode("{[1,3),[7,10)}").unwrap();
        let out = int8mr_nth(&[InValue::Int(0), InValue::Custom(bytes.as_slice())]);
        assert!(matches!(out, VdfReturn::Binary(_)));
    }

    #[test]
    fn int8mr_nth_returns_last_component() {
        let bytes = int8mr_encode("{[1,3),[7,10)}").unwrap();
        let out = int8mr_nth(&[InValue::Int(1), InValue::Custom(bytes.as_slice())]);
        assert!(matches!(out, VdfReturn::Binary(_)));
    }

    #[test]
    fn int8mr_nth_out_of_range_returns_null() {
        let bytes = int8mr_encode("{[1,3)}").unwrap();
        assert!(matches!(
            int8mr_nth(&[InValue::Int(1), InValue::Custom(bytes.as_slice())]),
            VdfReturn::Null
        ));
    }

    #[test]
    fn int4mr_nth_returns_component() {
        let bytes = int4mr_encode("{[1,3),[7,10)}").unwrap();
        assert!(matches!(
            int4mr_nth(&[InValue::Int(0), InValue::Custom(bytes.as_slice())]),
            VdfReturn::Binary(_)
        ));
    }

    #[test]
    fn datemr_nth_returns_component() {
        let bytes = datemr_encode("{[2026-01-01,2026-01-03),[2026-01-07,2026-01-10)}").unwrap();
        assert!(matches!(
            datemr_nth(&[InValue::Int(1), InValue::Custom(bytes.as_slice())]),
            VdfReturn::Binary(_)
        ));
    }

    #[test]
    fn dtmr_nth_returns_component() {
        let bytes = dtmr_encode(
            "{[2026-01-01 00:00:00,2026-01-03 00:00:00),[2026-01-07 00:00:00,2026-01-10 00:00:00)}",
        )
        .unwrap();
        assert!(matches!(
            dtmr_nth(&[InValue::Int(0), InValue::Custom(bytes.as_slice())]),
            VdfReturn::Binary(_)
        ));
    }
}

/// Slice 3c: multirange aggregation (`RANGE_AGG`).
mod slice4 {
    use villagesql::{InValue, VdfReturn};
    use vsql_ranger_arranger::func::range_agg::{
        MrAggState, datemr_range_agg_accumulate, datemr_range_agg_clear, datemr_range_agg_result,
        dtmr_range_agg_accumulate, dtmr_range_agg_clear, dtmr_range_agg_result,
        int4mr_range_agg_accumulate, int4mr_range_agg_clear, int4mr_range_agg_result,
        int8mr_range_agg_accumulate, int8mr_range_agg_clear, int8mr_range_agg_result,
    };
    use vsql_ranger_arranger::multirange_types::{
        datemr_encode, dtmr_encode, int4mr_encode, int8mr_encode, mr_decode_to_vec,
    };

    #[test]
    fn int8mr_range_agg_merges_components() {
        let mut state = MrAggState::default();
        int8mr_range_agg_clear(&mut state);
        int8mr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                int8mr_encode("{[1,3)}").unwrap().as_slice(),
            )],
        );
        int8mr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                int8mr_encode("{[7,10)}").unwrap().as_slice(),
            )],
        );
        let out = int8mr_range_agg_result(&state);
        assert!(matches!(out, VdfReturn::Binary(_)));
        let bytes = match out {
            VdfReturn::Binary(b) => b,
            _ => unreachable!(),
        };
        let comps =
            mr_decode_to_vec::<vsql_ranger_arranger::subtype::int8::Int8Ops>(&bytes).unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].lower, 1);
        assert_eq!(comps[0].upper, 3);
        assert_eq!(comps[1].lower, 7);
        assert_eq!(comps[1].upper, 10);
    }

    #[test]
    fn int4mr_range_agg_merges_components() {
        let mut state = MrAggState::default();
        int4mr_range_agg_clear(&mut state);
        int4mr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                int4mr_encode("{[1,3)}").unwrap().as_slice(),
            )],
        );
        int4mr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                int4mr_encode("{[7,10)}").unwrap().as_slice(),
            )],
        );
        let out = int4mr_range_agg_result(&state);
        assert!(matches!(out, VdfReturn::Binary(_)));
        let bytes = match out {
            VdfReturn::Binary(b) => b,
            _ => unreachable!(),
        };
        let comps =
            mr_decode_to_vec::<vsql_ranger_arranger::subtype::int4::Int4Ops>(&bytes).unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].lower, 1);
        assert_eq!(comps[0].upper, 3);
        assert_eq!(comps[1].lower, 7);
        assert_eq!(comps[1].upper, 10);
    }

    #[test]
    fn datemr_range_agg_merges_components() {
        let mut state = MrAggState::default();
        datemr_range_agg_clear(&mut state);
        datemr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                datemr_encode("{[2026-01-01,2026-01-03)}")
                    .unwrap()
                    .as_slice(),
            )],
        );
        datemr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                datemr_encode("{[2026-01-07,2026-01-10)}")
                    .unwrap()
                    .as_slice(),
            )],
        );
        let out = datemr_range_agg_result(&state);
        assert!(matches!(out, VdfReturn::Binary(_)));
        let bytes = match out {
            VdfReturn::Binary(b) => b,
            _ => unreachable!(),
        };
        let comps =
            mr_decode_to_vec::<vsql_ranger_arranger::subtype::date::DateOps>(&bytes).unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].lower, 20454);
        assert_eq!(comps[0].upper, 20456);
        assert_eq!(comps[1].lower, 20460);
        assert_eq!(comps[1].upper, 20463);
    }

    #[test]
    fn dtmr_range_agg_merges_components() {
        let mut state = MrAggState::default();
        dtmr_range_agg_clear(&mut state);
        dtmr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                dtmr_encode("{[2026-01-01 00:00:00,2026-01-03 00:00:00)}")
                    .unwrap()
                    .as_slice(),
            )],
        );
        dtmr_range_agg_accumulate(
            &mut state,
            &[InValue::Custom(
                dtmr_encode("{[2026-01-07 00:00:00,2026-01-10 00:00:00)}")
                    .unwrap()
                    .as_slice(),
            )],
        );
        let out = dtmr_range_agg_result(&state);
        assert!(matches!(out, VdfReturn::Binary(_)));
        let bytes = match out {
            VdfReturn::Binary(b) => b,
            _ => unreachable!(),
        };
        let comps =
            mr_decode_to_vec::<vsql_ranger_arranger::subtype::datetime::DateTimeOps>(&bytes)
                .unwrap();
        assert_eq!(comps.len(), 2);
        assert_eq!(comps[0].lower, 1767225600000000);
        assert_eq!(comps[0].upper, 1767398400000000);
        assert_eq!(comps[1].lower, 1767744000000000);
        assert_eq!(comps[1].upper, 1768003200000000);
    }
}
