#[cfg(feature = "slotmap")]
mod slotmap_impl {
    use crate::{known_deep_size, Context, DeepSizeOf};
    use core::mem::size_of;

    known_deep_size!(0; slotmap::KeyData, slotmap::DefaultKey);

    impl<K, V> DeepSizeOf for slotmap::SlotMap<K, V>
    where
        K: DeepSizeOf + slotmap::Key,
        V: DeepSizeOf + slotmap::Slottable,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            self.iter().fold(0, |sum, (key, val)| {
                sum + key.deep_size_of_children(context) + val.deep_size_of_children(context)
            }) + self.capacity() * size_of::<(u32, V)>()
        }
    }
}

#[cfg(feature = "slab")]
mod slab_impl {
    use crate::{Context, DeepSizeOf};
    use core::mem::size_of;

    // Mirror's `slab`'s internal `Entry` struct
    enum MockEntry<T> {
        _Vacant(usize),
        _Occupied(T),
    }

    impl<T> DeepSizeOf for slab::Slab<T>
    where
        T: DeepSizeOf,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            let capacity_size = self.capacity() * size_of::<MockEntry<T>>();
            let owned_size = self
                .iter()
                .fold(0, |sum, (_, val)| sum + val.deep_size_of_children(context));
            capacity_size + owned_size
        }
    }
}

#[cfg(feature = "arrayvec")]
mod arrayvec_impl {
    use crate::{known_deep_size, Context, DeepSizeOf};

    impl<A> DeepSizeOf for arrayvec::ArrayVec<A>
    where
        A: arrayvec::Array,
        <A as arrayvec::Array>::Item: DeepSizeOf,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            self.iter()
                .fold(0, |sum, elem| sum + elem.deep_size_of_children(context))
        }
    }

    known_deep_size!(0; { A: arrayvec::Array<Item=u8> + Copy } arrayvec::ArrayString<A>);
}

#[cfg(feature = "smallvec")]
mod smallvec_impl {
    use crate::{Context, DeepSizeOf};
    use core::mem::size_of;

    impl<A> DeepSizeOf for smallvec::SmallVec<A>
    where
        A: smallvec::Array,
        <A as smallvec::Array>::Item: DeepSizeOf,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            let child_size = self
                .iter()
                .fold(0, |sum, elem| sum + elem.deep_size_of_children(context));
            if self.spilled() {
                child_size + self.capacity() * size_of::<<A as smallvec::Array>::Item>()
            } else {
                child_size
            }
        }
    }
}

#[cfg(feature = "hashbrown")]
mod hashbrown_impl {
    use crate::{Context, DeepSizeOf};
    use core::mem::size_of;

    // This is probably still incorrect, but it's better than before
    impl<K, V, S> DeepSizeOf for hashbrown::HashMap<K, V, S>
    where
        K: DeepSizeOf + Eq + std::hash::Hash,
        V: DeepSizeOf,
        S: std::hash::BuildHasher,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            self.iter().fold(0, |sum, (key, val)| {
                sum + key.deep_size_of_children(context) + val.deep_size_of_children(context)
            }) + self.capacity() * size_of::<(K, V)>()
            // Buckets would be the more correct value, but there isn't
            // an API for accessing that with hashbrown.
            // I believe that hashbrown's HashTable is represented as
            // an array of (K, V), with control bytes at the start/end
            // that mark used/uninitialized buckets (?)
        }
    }

    impl<K, S> DeepSizeOf for hashbrown::HashSet<K, S>
    where
        K: DeepSizeOf + Eq + std::hash::Hash,
        S: std::hash::BuildHasher,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            self.iter()
                .fold(0, |sum, key| sum + key.deep_size_of_children(context))
                + self.capacity() * size_of::<K>()
        }
    }
}

#[cfg(feature = "indexmap")]
mod indexmap_impl {
    use crate::{Context, DeepSizeOf};
    use core::mem::size_of;
    use indexmap::{IndexMap, IndexSet};

    // IndexMap uses a vec of buckets (usize, K, V) as backing, with
    // a hashbrown::RawTable<usize> for lookups.  This method will
    // consistently underestimate, because IndexMap::capacity will
    // return the min of the capacity of the buckets list and the
    // capacity of the raw table.
    impl<K, V, S> DeepSizeOf for IndexMap<K, V, S>
    where
        K: DeepSizeOf,
        V: DeepSizeOf,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            let child_sizes = self.iter().fold(0, |sum, (key, val)| {
                sum + key.deep_size_of_children(context) + val.deep_size_of_children(context)
            });
            let map_size = self.capacity() * (size_of::<(usize, K, V)>() + size_of::<usize>());
            child_sizes + map_size
        }
    }
    impl<K, S> DeepSizeOf for IndexSet<K, S>
    where
        K: DeepSizeOf,
    {
        fn deep_size_of_children(&self, context: &mut Context) -> usize {
            let child_sizes = self
                .iter()
                .fold(0, |sum, key| sum + key.deep_size_of_children(context));
            let map_size = self.capacity() * (size_of::<(usize, K, ())>() + size_of::<usize>());
            child_sizes + map_size
        }
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use crate::known_deep_size;
    use chrono::*;

    known_deep_size!(0;
        NaiveDate, NaiveTime, NaiveDateTime, IsoWeek,
        Duration, Month, Weekday,
        FixedOffset, Local, Utc,
        {T: TimeZone} DateTime<T>, {T: TimeZone} Date<T>,
    );
}

#[cfg(feature = "tokio_net")]
mod tokio_net_impl {
    use crate::known_deep_size;
    use tokio::net::{TcpListener, TcpStream, UdpSocket, UnixDatagram, UnixListener, UnixStream};

    known_deep_size!(0;
        TcpListener, TcpStream, UdpSocket,
        UnixDatagram, UnixListener, UnixStream
    );
}

#[cfg(feature = "actix")]
mod actix_impl {
    use crate::{Context, DeepSizeOf};
    use actix::Addr;

    impl<T: actix::Actor> DeepSizeOf for Addr<T> {
        fn deep_size_of_children(&self, _context: &mut Context) -> usize {
            0
        }
    }
}

#[cfg(feature = "cpe")]
mod cpe_impl {
    use crate::{Context, DeepSizeOf};
    use cpe::component::OwnedComponent;
    use cpe::cpe::{Cpe, Language};
    use cpe::uri::OwnedUri;

    impl DeepSizeOf for OwnedComponent {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            if let OwnedComponent::Value(v) = self {
                v.deep_size_of_children(ctx)
            } else {
                0
            }
        }
    }

    impl DeepSizeOf for Language {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            if let Language::Language(v) = self {
                v.as_str().deep_size_of_children(ctx)
            } else {
                0
            }
        }
    }

    impl DeepSizeOf for OwnedUri {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            self.vendor().to_owned().deep_size_of_children(ctx)
                + self.product().to_owned().deep_size_of_children(ctx)
                + self.version().to_owned().deep_size_of_children(ctx)
                + self.update().to_owned().deep_size_of_children(ctx)
                + self.edition().to_owned().deep_size_of_children(ctx)
                + self.sw_edition().to_owned().deep_size_of_children(ctx)
                + self.target_sw().to_owned().deep_size_of_children(ctx)
                + self.other().to_owned().deep_size_of_children(ctx)
                + self.language().to_owned().deep_size_of_children(ctx)
        }
    }
}

#[cfg(feature = "petgraph")]
mod petgraph_impl {
    use crate::{Context, DeepSizeOf};
    use petgraph::graph::{Edge, Graph, Node};

    impl<N: DeepSizeOf> DeepSizeOf for Node<N> {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            self.weight.deep_size_of_children(ctx)
        }
    }

    impl<E: DeepSizeOf> DeepSizeOf for Edge<E> {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            self.weight.deep_size_of_children(ctx)
        }
    }

    impl<N: DeepSizeOf, E: DeepSizeOf> DeepSizeOf for Graph<N, E> {
        fn deep_size_of_children(&self, ctx: &mut Context) -> usize {
            self.raw_nodes().deep_size_of_children(ctx)
                + self.raw_edges().deep_size_of_children(ctx)
        }
    }
}
