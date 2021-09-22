use syn::{
    parse::{Parse, ParseStream, Result},
    Attribute, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, Token,
};

#[derive(Clone)]
pub enum Item {
    Trait(ItemTrait),
    Struct(ItemStruct),
    Enum(ItemEnum),
    Impl(ItemImpl),
    Fn(ItemFn),
}
impl Parse for Item {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Token![unsafe]) {
            let ahead = input.fork();
            ahead.parse::<Token![unsafe]>()?;
            lookahead = ahead.lookahead1();
        }
        if lookahead.peek(Token![impl]) {
            let mut item: ItemImpl = input.parse()?;
            item.attrs = attrs;
            Ok(Item::Impl(item))
        } else if lookahead.peek(Token![pub])
            || lookahead.peek(Token![trait])
            || lookahead.peek(Token![fn])
            || lookahead.peek(Token![async])
            || lookahead.peek(Token![enum])
            || lookahead.peek(Token![struct])
        {
            if lookahead.peek(Token![pub]) {
                let ahead = input.fork();
                ahead.parse::<Token![pub]>()?;
                lookahead = ahead.lookahead1();
            }
            if lookahead.peek(Token![trait]) {
                let mut item: ItemTrait = input.parse()?;
                item.attrs = attrs;
                Ok(Item::Trait(item))
            } else if lookahead.peek(Token![enum]) {
                let mut item: ItemEnum = input.parse()?;
                item.attrs = attrs;
                Ok(Item::Enum(item))
            } else if lookahead.peek(Token![struct]) {
                let mut item: ItemStruct = input.parse()?;
                item.attrs = attrs;
                Ok(Item::Struct(item))
            } else {
                let mut item: ItemFn = input.parse()?;
                item.attrs = attrs;
                Ok(Item::Fn(item))
            }
        } else {
            Err(lookahead.error())
        }
    }
}
