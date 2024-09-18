pub struct Wrap<'a, T>(pub &'a T);
pub unsafe fn extend<'b, T>(r: Wrap<'b, T>) -> Wrap<'static, T> {
    std::mem::transmute::<Wrap<'b, T>, Wrap<'static, T>>(r)
}

pub struct MutWrap<'a, T>(pub &'a mut T);
pub unsafe fn extend_mut<'b, T>(r: MutWrap<'b, T>) -> MutWrap<'static, T> {
    std::mem::transmute::<MutWrap<'b, T>, MutWrap<'static, T>>(r)
}
