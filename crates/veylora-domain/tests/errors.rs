use veylora_domain::errors::{ErrorCategory, ErrorCode};
use veylora_domain::preflight::{Finding, Severity, Validate};

#[test]
fn error_codes_have_stable_identifiers_categories_and_actionable_messages() {
    let cases = [
        (ErrorCode::MissingAsset, "missing_asset", ErrorCategory::Input, "Restore the missing source asset."),
        (ErrorCode::UnsupportedInput, "unsupported_input", ErrorCategory::Input, "Use a supported input format and metadata policy."),
        (ErrorCode::UnsupportedProfile, "unsupported_profile", ErrorCategory::Color, "Select a supported color profile."),
        (ErrorCode::UnverifiedDeviceSetup, "unverified_device_setup", ErrorCategory::Device, "Verify the device setup with physical channel evidence."),
        (ErrorCode::IncompatibleRecipe, "incompatible_recipe", ErrorCategory::Device, "Select a recipe compatible with the device setup and print mode."),
        (ErrorCode::UnsupportedControl, "unsupported_control", ErrorCategory::Device, "Remove the control that the backend cannot honor."),
        (ErrorCode::Clipping, "clipping", ErrorCategory::Layout, "Move or resize artwork to fit the printable bounds."),
        (ErrorCode::LowEffectiveResolution, "low_effective_resolution", ErrorCategory::Layout, "Use higher-resolution artwork or reduce the printed size."),
        (ErrorCode::UnsafeMargin, "unsafe_margin", ErrorCategory::Layout, "Move artwork inside the safe margin."),
        (ErrorCode::UnsupportedMediaSize, "unsupported_media_size", ErrorCategory::Layout, "Use a page size the device setup supports."),
        (ErrorCode::CoverageLimitExceeded, "coverage_limit_exceeded", ErrorCategory::Separation, "Reduce coverage to the validated ink and wet-load limits."),
        (ErrorCode::InsufficientDiskSpace, "insufficient_disk_space", ErrorCategory::Resource, "Free disk space before rendering or retaining artifacts."),
        (ErrorCode::AmbiguousSubmission, "ambiguous_submission", ErrorCategory::Submission, "Review the physical output before creating another submission attempt. Do not retry automatically."),
    ];
    for (code, identifier, category, message) in cases {
        assert_eq!(code.as_str(), identifier);
        assert_eq!(code.to_string(), identifier);
        assert_eq!(code.category(), category);
        assert_eq!(code.message(), message);
        assert_eq!(serde_json::to_value(code).unwrap(), identifier);
        assert_eq!(
            serde_json::from_value::<ErrorCode>(serde_json::json!(identifier)).unwrap(),
            code
        );
    }
}

#[test]
fn validation_findings_preserve_subject_message_code_and_severity() {
    struct MissingArtwork;
    impl Validate for MissingArtwork {
        fn validate(&self) -> Vec<Finding> {
            vec![Finding {
                code: ErrorCode::MissingAsset,
                severity: Severity::Blocking,
                message: "Restore artwork.png from the retained source bundle.".into(),
                subject: Some("artwork.png".into()),
            }]
        }
    }
    let validator: &dyn Validate = &MissingArtwork;
    let findings = validator.validate();
    let wire = serde_json::json!([{
        "code": "missing_asset",
        "severity": "blocking",
        "message": "Restore artwork.png from the retained source bundle.",
        "subject": "artwork.png"
    }]);
    assert_eq!(serde_json::to_value(&findings).unwrap(), wire);
    assert_eq!(
        serde_json::from_value::<Vec<Finding>>(wire).unwrap(),
        findings
    );
}

#[test]
fn severity_and_categories_have_stable_wire_names() {
    for (severity, name) in [
        (Severity::Blocking, "blocking"),
        (Severity::ConfirmationRequired, "confirmation_required"),
        (Severity::Informational, "informational"),
    ] {
        let finding = Finding {
            code: ErrorCode::UnverifiedDeviceSetup,
            severity,
            message: ErrorCode::UnverifiedDeviceSetup.message().into(),
            subject: None,
        };
        let wire = serde_json::to_value(&finding).unwrap();
        assert_eq!(wire["severity"], name);
        assert_eq!(wire["subject"], serde_json::Value::Null);
        assert_eq!(serde_json::from_value::<Finding>(wire).unwrap(), finding);
    }
    for (category, name) in [
        (ErrorCategory::Input, "input"),
        (ErrorCategory::Color, "color"),
        (ErrorCategory::Device, "device"),
        (ErrorCategory::Layout, "layout"),
        (ErrorCategory::Separation, "separation"),
        (ErrorCategory::Resource, "resource"),
        (ErrorCategory::Submission, "submission"),
    ] {
        assert_eq!(serde_json::to_value(category).unwrap(), name);
        assert_eq!(
            serde_json::from_value::<ErrorCategory>(serde_json::json!(name)).unwrap(),
            category
        );
    }
}

#[test]
fn unknown_codes_and_severities_are_rejected_not_downgraded() {
    assert!(serde_json::from_str::<ErrorCode>("\"unknown_code\"").is_err());
    assert!(serde_json::from_str::<ErrorCategory>("\"unknown_category\"").is_err());
    assert!(serde_json::from_str::<Severity>("\"warning\"").is_err());
}

#[test]
fn severity_distinguishes_hard_stops_confirmation_and_information() {
    assert!(Severity::Blocking.is_blocking());
    assert!(!Severity::Blocking.requires_confirmation());
    assert!(!Severity::ConfirmationRequired.is_blocking());
    assert!(Severity::ConfirmationRequired.requires_confirmation());
    assert!(!Severity::Informational.is_blocking());
    assert!(!Severity::Informational.requires_confirmation());
}
