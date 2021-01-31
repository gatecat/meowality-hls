use crate::design::Context;


// This is for converting objects to strings with the context 
// TODO: ways of avoiding an unnecessary copy?
pub trait ToStrWithCtx {
	fn to_str_with_ctx(&self, ctx: &Context) -> String;
}
