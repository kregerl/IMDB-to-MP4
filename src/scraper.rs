use scraper::{Html, Selector};

use crate::vidsrc::{VidsrcError, VidsrcResult};

pub fn get_document<'a>(url: &str) -> VidsrcResult<'a, Html> {
    let response = reqwest::blocking::get(url)?;
    let html = response.text()?;
    Ok(scraper::Html::parse_document(&html).to_owned())
}

pub fn parse_attribute<'a>(
    document: &Html,
    selector_str: &'a str,
    attr: &str,
) -> VidsrcResult<'a, String> {
    let selector = Selector::parse(selector_str)?;
    let tag = document
        .select(&selector)
        .next()
        .ok_or(VidsrcError::EmptySelector)?;
    let atribute = tag.attr(attr).ok_or(VidsrcError::EmptyAttr)?;
    Ok(atribute.into())
}

pub fn parse_inner_html<'a>(document: &Html, selector_str: &'a str) -> VidsrcResult<'a, String> {
    let selector = Selector::parse(selector_str)?;
    let tag = document
        .select(&selector)
        .next()
        .ok_or(VidsrcError::EmptySelector)?;
    Ok(tag.inner_html())
}