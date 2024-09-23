/**
When `self.len` <= FINAL:
> Adds zeroes to the left side of `self` until `length = FINAL`.

When `self.len` > FINAL:
> Removes elements from the left side of `self` until `length = FINAL`.

Consumes the input and outputs a new [u8] array.
*/
#[trait_variant::make(Send)]
pub trait Left: Sized + Sync + IntoIterator<Item = u8>
where
    <Self as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    async fn pad_left<const FINAL: usize>(self) -> [u8; FINAL] {
        async {
            let mut r: [u8; FINAL] = [0; FINAL];
            let mut iterator = self.into_iter();

            for index in (0..FINAL).rev() {
                if let Some(byte) = iterator.next() {
                    r[index] = byte;
                } else {
                    r[index] = 0;
                }
            }

            r
        }
    }
}
impl<const INITIAL: usize> Left for [u8; INITIAL] {}
impl Left for Vec<u8> {}

/**
When `self.len` <= FINAL:
> Adds zeroes to the right side of `self` until `length = FINAL`.

When `self.len` > FINAL:
> Removes elements from the right side of `self` until `length = FINAL`.

Consumes the input and outputs a new [u8] array.
*/
#[trait_variant::make(Send)]
pub trait Right: Sized + Sync + IntoIterator<Item = u8> {
    async fn pad_right<const FINAL: usize>(self) -> [u8; FINAL] {
        async {
            let mut r: [u8; FINAL] = [0; FINAL];

            let mut iterator = self.into_iter();
            for index in 0..FINAL {
                if let Some(byte) = iterator.next() {
                    r[index] = byte
                } else {
                    r[index] = 0
                }
            }
            r
        }
    }
}
impl<const INITIAL: usize> Right for [u8; INITIAL] {}
impl Right for Vec<u8> {}
