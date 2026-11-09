use wetext_rs::plate::PlateProfile;

fn profile() -> PlateProfile {
    PlateProfile::structured_plate4(include_str!("../profiles/number_plate_v1_5/lexicon.json"))
        .unwrap()
}

#[test]
fn default_plate4_accepts_confirmed_formats_and_same_input_corrections() {
    let mut engine = profile();
    for (raw, expected) in [
        ("苏B三个六七八", "苏B66678"),
        ("苏B一二三四五，不对，最后一位是六", "苏B12346"),
        ("沪建E幺零T五", "沪JE10T5"),
        ("嗯，那个新界E幺六T七。", "新JE16T7"),
        ("黑翼幺四四九Z", "黑E1449Z"),
        ("京圈E33H4", "京QE33H4"),
        ("川两个勾六G幺二", "川JJ6G12"),
        ("京A ABCDE的E一二三四", "京AE1234"),
        ("黑F零七二八警", "黑F0728警"),
        ("冀A幺二三四挂", "冀A1234挂"),
        ("粤Z幺二三四港", "粤Z1234港"),
        ("闽DD568学", "闽DD568学"),
        ("京A12345，警察过来了", "京A12345"),
        ("车牌京A12345，在小区转了两圈", "京A12345"),
    ] {
        let result = engine.normalize(raw).unwrap();
        assert_eq!(result.plate.as_deref(), Some(expected), "{raw}");
        assert_eq!(result.version, "plate_semantic_v1_6");
    }
}

#[test]
fn default_plate4_never_infers_business_context_or_remembers_previous_inputs() {
    let mut engine = profile();
    for raw in [
        "WJ京A12345",
        "京A12345警",
        "苏A12345挂",
        "应急京A1234",
        "京A1234试",
        "沪A5678超",
        "GB1S192",
        "沪AGN9808",
        "沪C零圈102",
        "浙M",
        "0F77K",
        "不对，最后一位是六",
        "",
    ] {
        assert_eq!(
            engine.normalize("苏B一二三四五").unwrap().plate.as_deref(),
            Some("苏B12345")
        );
        assert!(engine.normalize(raw).unwrap().plates.is_empty(), "{raw}");
    }
    let multi = engine.normalize("京A12345和苏B67890").unwrap();
    assert_eq!(multi.plate, None);
    assert_eq!(multi.plates, ["京A12345", "苏B67890"]);
}

#[test]
fn old_profile_keeps_its_rule_identity_and_behavior() {
    let lexicon = include_str!("../profiles/number_plate_v1_5/lexicon.json");
    let mut old = PlateProfile::structured(lexicon).unwrap();
    assert_eq!(
        old.normalize("WJ京A12345").unwrap().plate.as_deref(),
        Some("京A12345")
    );
    assert_eq!(old.normalize("黑F零七二八警").unwrap().plate, None);
    assert_eq!(old.parameters()["semantic_version"], "plate_semantic_v1_5");
    assert_eq!(
        profile().parameters()["semantic_version"],
        "plate_semantic_v1_6"
    );
}
