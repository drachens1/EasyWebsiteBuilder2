use crate::page::PageType;

pub struct DynamicPage {
  path: String,
  gen_html: Box<dyn Fn() -> Vec<u8> + Send + Sync>,
  page_type: PageType,
  etag: String,
}
impl DynamicPage {

}
