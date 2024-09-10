pub struct Wrap<'a, T>(pub &'a T);

pub unsafe fn extend<'b, T>(r: Wrap<'b, T>) -> Wrap<'static, T> {
    std::mem::transmute::<Wrap<'b, T>, Wrap<'static, T>>(r)
}
