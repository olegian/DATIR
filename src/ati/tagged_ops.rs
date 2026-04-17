use crate::ati::{ati::ATI_ANALYSIS, tagged::Tagged};

// =====================    COMPARISON OPS / MARKERS    ===================
/// A Tagged<T> is PartialEq iff T is PartialEq
impl<T> std::cmp::PartialEq for Tagged<T>
where
    T: std::cmp::PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &other.0);
        self.1.eq(&other.1)
    }
}

/// A Tagged<T> is PartialOrd iff T is PartialOrd
impl<T> std::cmp::PartialOrd for Tagged<T>
where
    T: std::cmp::PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &other.0);
        self.1.partial_cmp(&other.1)
    }
}

/// A Tagged<T> is Eq if T is Eq
impl<T> std::cmp::Eq for Tagged<T> where T: std::cmp::Eq {}

/// A Tagged<T> is Ord if T is Ord
impl<T> std::cmp::Ord for Tagged<T>
where
    T: std::cmp::Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        ATI_ANALYSIS
            .lock()
            .unwrap()
            .union_and_get_id(&self.0, &other.0);
        self.1.cmp(&other.1)
    }
}

// =====================    ARITHEMATIC OPS    ===================
// all of these operators merge together the tags of the result, lhs, and rhs.
macro_rules! impl_tagged_arithematic_op {
    (
        $trait:ident,
        $method:ident,
        $assign_trait:ident,
        $assign_method:ident,
        $op:tt
    ) => {
        impl<T> std::ops::$trait for Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Self) -> Self::Output {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                Tagged(merged, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait<&Tagged<T>> for Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: &Tagged<T>) -> Self::Output {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                Tagged(merged, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait for &Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Self) -> Self::Output {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                Tagged(merged, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait<Tagged<T>> for &Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Tagged<T>) -> Self::Output {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                Tagged(merged, self.1 $op rhs.1)
            }
        }

        impl<T> std::ops::$assign_trait for Tagged<T>
        where
            T: std::ops::$trait<Output = T> + Copy,
        {
            fn $assign_method(&mut self, rhs: Self) {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                *self = Tagged(merged, self.1 $op rhs.1);
            }
        }

        impl<T: Copy> std::ops::$assign_trait<&Tagged<T>> for Tagged<T>
        where
            T: std::ops::$trait<Output = T> + Copy,
        {
            fn $assign_method(&mut self, rhs: &Tagged<T>) {
                let merged = ATI_ANALYSIS
                    .lock()
                    .unwrap()
                    .union_and_get_id(&self.0, &rhs.0);
                *self = Tagged(merged, self.1 $op rhs.1);
            }
        }
    };
}

impl_tagged_arithematic_op!(Add, add, AddAssign, add_assign, +);
impl_tagged_arithematic_op!(Sub, sub, SubAssign, sub_assign, -);
impl_tagged_arithematic_op!(Mul, mul, MulAssign, mul_assign, *);
impl_tagged_arithematic_op!(Div, div, DivAssign, div_assign, /);
impl_tagged_arithematic_op!(Rem, rem, RemAssign, rem_assign, %);
impl_tagged_arithematic_op!(BitAnd, bitand, BitAndAssign, bitand_assign, &);
impl_tagged_arithematic_op!(BitOr,  bitor,  BitOrAssign,  bitor_assign, |);
impl_tagged_arithematic_op!(BitXor, bitxor, BitXorAssign, bitxor_assign, ^);

// =====================    SHIFT OPS    ===================
// these operators merge together the tags of the lhs and the result, but not
// the rhs.
macro_rules! impl_tagged_shift_op {
    (
        $trait:ident,
        $method:ident,
        $assign_trait:ident,
        $assign_method:ident,
        $op:tt
    ) => {
        impl<T> std::ops::$trait for Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Self) -> Self::Output {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                Tagged(new_id, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait<&Tagged<T>> for Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: &Tagged<T>) -> Self::Output {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                Tagged(new_id, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait for &Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Self) -> Self::Output {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                Tagged(new_id, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$trait<Tagged<T>> for &Tagged<T>
        where
            T: std::ops::$trait<Output = T>,
        {
            type Output = Tagged<T>;
            fn $method(self, rhs: Tagged<T>) -> Self::Output {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                Tagged(new_id, self.1 $op rhs.1)
            }
        }

        impl<T> std::ops::$assign_trait for Tagged<T>
        where
            T: std::ops::$trait<Output = T> + Copy,
        {
            fn $assign_method(&mut self, rhs: Self) {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                *self = Tagged(new_id, self.1 $op rhs.1)
            }
        }

        impl<T: Copy> std::ops::$assign_trait<&Tagged<T>> for Tagged<T>
        where
            T: std::ops::$trait<Output = T> + Copy,
        {
            fn $assign_method(&mut self, rhs: &Tagged<T>) {
                let new_id = ATI_ANALYSIS.lock().unwrap().make_id();
                ATI_ANALYSIS.lock().unwrap().union_and_get_id(&new_id, &self.0);
                *self = Tagged(new_id, self.1 $op rhs.1)
            }
        }
    };
}

impl_tagged_shift_op!(Shl, shl, ShlAssign, shl_assign, <<);
impl_tagged_shift_op!(Shr, shr, ShrAssign, shr_assign, >>);

// =====================    UNARY OPS    ===================
// these operators just get pushed down to act on the underlying value.
impl<T> std::ops::Neg for Tagged<T>
where
    T: std::ops::Neg<Output = T>,
{
    type Output = Tagged<T>;

    fn neg(self) -> Self::Output {
        Tagged(self.0, -self.1)
    }
}

impl<T> std::ops::Not for Tagged<T>
where
    T: std::ops::Not<Output = T>,
{
    type Output = Tagged<T>;
    fn not(self) -> Self::Output {
        Tagged(self.0, !self.1)
    }
}

// this is a really important impl! This gets used for deref coercion,
// which allows for a &Tagged<T> to automatically be coereced to &T,
// which allows for dispatching methods that are defined on T using a 
// Tagged<T>.
impl<T> std::ops::Deref for Tagged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

