(function() {var implementors = {
"axalloc":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axalloc/struct.GlobalPage.html\" title=\"struct axalloc::GlobalPage\">GlobalPage</a>"]],
"axfs":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axfs/fops/struct.Directory.html\" title=\"struct axfs::fops::Directory\">Directory</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axfs/fops/struct.File.html\" title=\"struct axfs::fops::File\">File</a>"]],
"axnet":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axnet/struct.UdpSocket.html\" title=\"struct axnet::UdpSocket\">UdpSocket</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axnet/struct.TcpSocket.html\" title=\"struct axnet::TcpSocket\">TcpSocket</a>"]],
"axstd":[["impl&lt;'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axstd/sync/struct.MutexGuard.html\" title=\"struct axstd::sync::MutexGuard\">MutexGuard</a>&lt;'a, T&gt;"]],
"axsync":[["impl&lt;'a, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axsync/struct.MutexGuard.html\" title=\"struct axsync::MutexGuard\">MutexGuard</a>&lt;'a, T&gt;"]],
"axtask":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"axtask/struct.TaskInner.html\" title=\"struct axtask::TaskInner\">TaskInner</a>"]],
"driver_net":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"driver_net/struct.NetBuf.html\" title=\"struct driver_net::NetBuf\">NetBuf</a>"]],
"kernel_guard":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"kernel_guard/struct.NoPreemptIrqSave.html\" title=\"struct kernel_guard::NoPreemptIrqSave\">NoPreemptIrqSave</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"kernel_guard/struct.NoPreempt.html\" title=\"struct kernel_guard::NoPreempt\">NoPreempt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"kernel_guard/struct.IrqSave.html\" title=\"struct kernel_guard::IrqSave\">IrqSave</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"kernel_guard/struct.NoOp.html\" title=\"struct kernel_guard::NoOp\">NoOp</a>"]],
"lazy_init":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"lazy_init/struct.LazyInit.html\" title=\"struct lazy_init::LazyInit\">LazyInit</a>&lt;T&gt;"]],
"linked_list":[["impl&lt;G: <a class=\"trait\" href=\"linked_list/trait.AdapterWrapped.html\" title=\"trait linked_list::AdapterWrapped\">AdapterWrapped</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"linked_list/struct.List.html\" title=\"struct linked_list::List\">List</a>&lt;G&gt;"]],
"page_table":[["impl&lt;M: <a class=\"trait\" href=\"page_table/trait.PagingMetaData.html\" title=\"trait page_table::PagingMetaData\">PagingMetaData</a>, PTE: <a class=\"trait\" href=\"page_table_entry/trait.GenericPTE.html\" title=\"trait page_table_entry::GenericPTE\">GenericPTE</a>, IF: <a class=\"trait\" href=\"page_table/trait.PagingIf.html\" title=\"trait page_table::PagingIf\">PagingIf</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"page_table/struct.PageTable64.html\" title=\"struct page_table::PageTable64\">PageTable64</a>&lt;M, PTE, IF&gt;"]],
"spinlock":[["impl&lt;'a, G: <a class=\"trait\" href=\"kernel_guard/trait.BaseGuard.html\" title=\"trait kernel_guard::BaseGuard\">BaseGuard</a>, T: ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/drop/trait.Drop.html\" title=\"trait core::ops::drop::Drop\">Drop</a> for <a class=\"struct\" href=\"spinlock/struct.BaseSpinLockGuard.html\" title=\"struct spinlock::BaseSpinLockGuard\">BaseSpinLockGuard</a>&lt;'a, G, T&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()