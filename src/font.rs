use iced::{
    Font,
    font::{Family, Weight},
};

pub(crate) const FIRA_CODE_FAMILY: &str = "Fira Code";

pub(crate) const FIRA_REGULAR_BYTES: &[u8] = include_bytes!("../ttf/FiraCode-Regular.ttf");
pub(crate) const FIRA_BOLD_BYTES: &[u8] = include_bytes!("../ttf/FiraCode-Bold.ttf");

pub(crate) const FIRA_REGULAR: Font = Font {
    family: Family::Name(FIRA_CODE_FAMILY),
    weight: Weight::Normal,
    ..Font::DEFAULT
};

pub(crate) const FIRA_BOLD: Font = Font {
    family: Family::Name(FIRA_CODE_FAMILY),
    weight: Weight::Bold,
    ..Font::DEFAULT
};
