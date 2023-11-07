#![allow(dead_code)]

fn raw_str<'s>(ptr: *const u8) -> &'s str {
    if ptr.is_null() {
        return "";
    }
    unsafe {
        let mut i = 0;
        while *ptr.add(i) != 0 {
            assert!(i < 512, "exceeded search limit for string end");
            i += 1;
        }
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, i))
    }
}

mod fstapi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    use std::convert::TryFrom;
    macro_rules! cenum {
        ($name:ident : $repr:tt , $($var:ident = $val:literal),* $(,)?) => {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            #[repr($repr)]
            pub enum $name {
                $($var = $val),*
            }
            impl TryFrom<$repr> for $name {
                type Error = std::num::TryFromIntError;

                fn try_from(value: $repr) -> Result<Self, Self::Error> {
                    use $name::*;
                    match value {
                        $(n if n == $var as $repr => Ok($var),)*
                        _ => Err(u8::try_from(256).err().unwrap()),
                    }
                }
            }

            impl TryFrom<u32> for $name {
                type Error = std::num::TryFromIntError;
                fn try_from(value: u32) -> Result<Self, Self::Error> {
                    let value: $repr = value.try_into()?;
                    Self::try_from(value)
                }
            }
        }
    }
    cenum! { ScopeType: u8,
        VcdModule = 0,
        VcdTask = 1,
        VcdFunction = 2,
        VcdBegin = 3,
        VcdFork = 4,
        VcdGenerate = 5,
        VcdStruct = 6,
        VcdUnion = 7,
        VcdClass = 8,
        VcdInterface = 9,
        VcdPackage = 10,
        VcdProgram = 11,

        VhdlArchitecture = 12,
        VhdlProcedure = 13,
        VhdlFunction = 14,
        VhdlRecord = 15,
        VhdlProcess = 16,
        VhdlBlock = 17,
        VhdlForGenerate = 18,
        VhdlIfGenerate = 19,
        VhdlGenerate = 20,
        VhdlPackage = 21,
    }

    cenum! { VarType: u8,
        VcdEvent = 0,
        VcdInteger = 1,
        VcdParameter = 2,
        VcdReal = 3,
        VcdRealParameter = 4,
        VcdReg = 5,
        VcdSupply0 = 6,
        VcdSupply1 = 7,
        VcdTime = 8,
        VcdTri = 9,
        VcdTriAnd = 10,
        VcdTriOr = 11,
        VcdTriReg = 12,
        VcdTri0 = 13,
        VcdTri1 = 14,
        VcdWand = 15,
        VcdWire = 16,
        VcdWor = 17,
        VcdPort = 18,
        VcdSpArray =
            19, /* used to define the rownum (index) port for a sparse array */
        VcdRealtime = 20,

        GenString =
            21, /* generic string type   (max len is defined dynamically via
                   fstWriterEmitVariableLengthValueChange) */

        SvBit = 22,
        SvLogic = 23,
        SvInt = 24,       /* declare as size = 32 */
        SvShortInt = 25,  /* declare as size = 16 */
        SvLongInt = 26,   /* declare as size = 64 */
        SvByte = 27,      /* declare as size = 8  */
        SvEnum = 28,      /* declare as appropriate type range */
        SvShortReal = 29, /* declare and emit same as FST_VT_VCD_REAL (needs
                                     to be emitted as double, not a float) */
    }

    cenum! { VarDir: u8,
        Implicit = 0,
        Input = 1,
        Output = 2,
        Inout = 3,
        Buffer = 4,
        Linkage = 5,
    }

    cenum! { MiscType: u8,
        Comment = 0, /* use fstwritersetcomment() to emit */
        EnvVar = 1,  /* use fstwritersetenvvar() to emit */
        SupVar = 2,  /* use fstwritercreatevar2() to emit */
        PathName =
            3, /* reserved for fstwritersetsourcestem() string -> number management */
        SourceStem = 4, /* use fstwritersetsourcestem() to emit */
        SourceIStem =
            5,                /* use fstwritersetsourceinstantiationstem() to emit */
        ValueList = 6, /* use fstwritersetvaluelist() to emit, followed by
                                 fstwritercreatevar*() */
        EnumTable = 7, /* use fstwritercreateenumtable() and
                                 fstwriteremitenumtableref() to emit */
        Unknown = 8,
    }

    cenum! { ArrayType: u8,
        None = 0,
        Unpacked = 1,
        Packed = 2,
        Sparse = 3,
    }

    cenum! { EnumValueType: u8,
        SvInteger = 0,
        SvBit = 1,
        SvLogic = 2,
        SvInt = 3,
        SvShortInt = 4,
        SvLongInt = 5,
        SvByte = 6,
        SvUnsignedInteger = 7,
        SvUnsignedBit = 8,
        SvUnsignedLogic = 9,
        SvUnsignedInt = 10,
        SvUnsignedShortInt = 11,
        SvUnsignedLongInt = 12,
        SvUnsignedByte = 13,

        Reg = 14,
        Time = 15,
    }

    cenum! { PackType: u8,
        None = 0,
        Unpacked = 1,
        Packed = 2,
        TaggedPacked = 3,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum AttrType {
        Misc(MiscType),
        Array(ArrayType),
        Enum(EnumValueType),
        Pack(PackType),
    }

    impl TryFrom<(u8, u8)> for AttrType {
        type Error = std::num::TryFromIntError;

        fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
            use AttrType::*;
            match value.0 {
                0 => Ok(Misc(value.1.try_into()?)),
                1 => Ok(Array(value.1.try_into()?)),
                2 => Ok(Enum(value.1.try_into()?)),
                3 => Ok(Pack(value.1.try_into()?)),
                _ => Err(u8::try_from(256).err().unwrap()),
            }
        }
    }

    cenum! { FileType: u8,
        Verilog = 0,
        Vhdl = 1,
        VerilogVhdl = 2,
    }

    type ReaderCtx = std::ffi::c_void;

    pub struct Reader {
        ctx: *mut ReaderCtx,
    }

    impl Reader {
        pub fn open(filename: &str) -> Option<Reader> {
            let mut nam = filename.to_string();
            nam.push('\0');
            let ctx = unsafe { fstReaderOpen(nam.as_ptr().cast()) };
            if ctx.is_null() {
                None
            } else {
                Some(Reader { ctx })
            }
        }

        pub fn hier_iter(&mut self) -> HierIter {
            HierIter { ctx: self.ctx }
        }

        pub fn iter_blocks(&mut self, mut f: impl FnMut(u64, fstHandle, &[u8])) {
            let mut userdata: &mut dyn FnMut(u64, fstHandle, &[u8]) = &mut f;
            let userdata: *mut &mut dyn FnMut(u64, fstHandle, &[u8]) = &mut userdata as *mut _;
            // SAFETY: userdata only needs to live the length of the fn call
            unsafe {
                fstReaderIterBlocks(
                    self.ctx,
                    Some(reader_iter_blocks_cb),
                    userdata.cast(),
                    std::ptr::null_mut(),
                )
            };
        }

        pub fn alias_count(&self) -> u64 {
            unsafe { fstReaderGetAliasCount(self.ctx) }
        }
        pub fn current_flat_scope(&self) -> &str {
            unsafe { super::raw_str(fstReaderGetCurrentFlatScope(self.ctx).cast()) }
        }
        //pub fn current_scope_user_info(&self) -> void *{ unsafe { fstReaderGetCurrentScopeUserInfo(self.ctx) } }
        pub fn current_scope_len(&self) -> usize {
            usize::try_from(unsafe { fstReaderGetCurrentScopeLen(self.ctx) }).unwrap()
        }
        pub fn date_string(&self) -> &str {
            unsafe { super::raw_str(fstReaderGetDateString(self.ctx).cast()) }
        }
        pub fn double_endian_match_state(&self) -> bool {
            unsafe { fstReaderGetDoubleEndianMatchState(self.ctx) != 0 }
        }
        pub fn dump_activity_change_time(&self, idx: u32) -> u64 {
            unsafe { fstReaderGetDumpActivityChangeTime(self.ctx, idx) }
        }
        pub fn dump_activity_change_value(&self, idx: u32) -> u8 {
            unsafe { fstReaderGetDumpActivityChangeValue(self.ctx, idx) }
        }
        pub fn end_time(&self) -> u64 {
            unsafe { fstReaderGetEndTime(self.ctx) }
        }
        pub fn fac_process_mask(&self, facidx: fstHandle) -> u32 {
            unsafe { fstReaderGetFacProcessMask(self.ctx, facidx) as u32 }
        }
        pub fn file_type(&self) -> FileType {
            u8::try_from(unsafe { fstReaderGetFileType(self.ctx) })
                .and_then(FileType::try_from)
                .unwrap()
        }
        pub fn fseek_failed(&self) -> bool {
            unsafe { fstReaderGetFseekFailed(self.ctx) != 0 }
        }
        pub fn max_handle(&self) -> fstHandle {
            unsafe { fstReaderGetMaxHandle(self.ctx) }
        }
        pub fn memory_used_by_writer(&self) -> u64 {
            unsafe { fstReaderGetMemoryUsedByWriter(self.ctx) }
        }
        pub fn number_dump_activity_changes(&self) -> u32 {
            unsafe { fstReaderGetNumberDumpActivityChanges(self.ctx) }
        }
        pub fn scope_count(&self) -> u64 {
            unsafe { fstReaderGetScopeCount(self.ctx) }
        }
        pub fn start_time(&self) -> u64 {
            unsafe { fstReaderGetStartTime(self.ctx) }
        }
        pub fn timescale(&self) -> i8 {
            unsafe { fstReaderGetTimescale(self.ctx) }
        }
        pub fn timezero(&self) -> i64 {
            unsafe { fstReaderGetTimezero(self.ctx) }
        }
        pub fn value_change_section_count(&self) -> u64 {
            unsafe { fstReaderGetValueChangeSectionCount(self.ctx) }
        }
        pub fn value_from_handle_at_time(&self, tim: u64, facidx: fstHandle, buf: *mut i8) -> &str {
            unsafe {
                super::raw_str(fstReaderGetValueFromHandleAtTime(self.ctx, tim, facidx, buf).cast())
            }
        }
        pub fn var_count(&self) -> u64 {
            unsafe { fstReaderGetVarCount(self.ctx) }
        }
        pub fn version_string(&self) -> &str {
            unsafe { super::raw_str(fstReaderGetVersionString(self.ctx).cast()) }
        }

        pub fn iter_values_at_time(&self, tim: u64, mut f: impl FnMut(fstHandle, &str)) {
            let mut buf: std::mem::MaybeUninit<[i8; 512]> = std::mem::MaybeUninit::uninit();
            for i in 0..self.max_handle() {
                f(
                    i,
                    self.value_from_handle_at_time(tim, i, buf.as_mut_ptr().cast()),
                );
            }
        }

        pub fn set_limit_time_range(&mut self, range: std::ops::Range<u64>) {
            unsafe { fstReaderSetLimitTimeRange(self.ctx, range.start, range.end) }
        }
        pub fn set_unlimited_time_range(&mut self) {
            unsafe { fstReaderSetUnlimitedTimeRange(self.ctx) }
        }
    }

    unsafe extern "C" fn reader_iter_blocks_cb(
        user_callback_data_pointer: *mut ::std::os::raw::c_void,
        time: u64,
        facidx: fstHandle,
        value: *const ::std::os::raw::c_uchar,
    ) {
        println!("callback {time:?} {facidx:?} {value:?}");
        let cb_f: *mut &mut dyn FnMut(u64, fstHandle, &[u8]) = user_callback_data_pointer.cast();
        let cb_f: &mut dyn FnMut(u64, fstHandle, &[u8]) = unsafe { *cb_f };
        let value: &[u8] = unsafe {
            let mut i = 0;
            while *value.add(i) != 0 {
                i += 1;
            }
            std::slice::from_raw_parts(value, i)
        };
        cb_f(time, facidx, value);
    }

    impl std::ops::Drop for Reader {
        fn drop(&mut self) {
            if !self.ctx.is_null() {
                unsafe { fstReaderClose(self.ctx) }
                self.ctx = std::ptr::null_mut();
            }
        }
    }

    pub enum Hier {
        Scope(fstHier__bindgen_ty_1_fstHierScope),
        Var(fstHier__bindgen_ty_1_fstHierVar),
        Attr(fstHier__bindgen_ty_1_fstHierAttr),
    }

    impl std::fmt::Debug for Hier {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fn raw_str<'s>(addr: *const i8, len: u32) -> &'s str {
                let s: &[u8] = unsafe { std::slice::from_raw_parts(addr.cast(), len as usize) };
                unsafe { std::str::from_utf8_unchecked(s) }
            }
            match self {
                Hier::Scope(s) => f
                    .debug_struct("Scope")
                    .field("typ", &ScopeType::try_from(s.typ).unwrap())
                    .field("name", &raw_str(s.name, s.name_length))
                    .field("component", &raw_str(s.component, s.component_length))
                    .finish(),
                Hier::Var(v) => f
                    .debug_struct("Var")
                    .field("typ", &VarType::try_from(v.typ).unwrap())
                    .field("direction", &VarDir::try_from(v.direction).unwrap())
                    .field("name", &raw_str(v.name, v.name_length))
                    .field("length", &v.length)
                    .field("handle", &v.handle)
                    .field("is_alias", &v.is_alias())
                    .finish(),
                Hier::Attr(a) => f
                    .debug_struct("Attr")
                    .field("typ", &AttrType::try_from((a.typ, a.subtype)).unwrap())
                    .field("name", &raw_str(a.name, a.name_length))
                    .field("arg", &a.arg)
                    .field("arg_from_name", &a.arg_from_name)
                    .finish(),
            }
        }
    }

    pub struct HierIter {
        ctx: *mut ReaderCtx,
    }

    impl std::iter::Iterator for HierIter {
        type Item = Hier;

        fn next(&mut self) -> Option<Self::Item> {
            if self.ctx.is_null() {
                return None;
            }
            // SAFETY: C ffi
            let hier = unsafe { fstReaderIterateHier(self.ctx) };
            if hier.is_null() {
                self.ctx = std::ptr::null_mut();
                return None;
            }
            // SAFETY: checked hier is not null
            match u32::from(unsafe { (*hier).htyp }) {
                fstHierType_FST_HT_SCOPE => Some(Hier::Scope(unsafe { (*hier).u.scope })),
                fstHierType_FST_HT_VAR => Some(Hier::Var(unsafe { (*hier).u.var })),
                fstHierType_FST_HT_ATTRBEGIN => Some(Hier::Attr(unsafe { (*hier).u.attr })),
                _ => None,
            }
        }
    }
}

fn main() {
    let mut fst = fstapi::Reader::open("test.fst").expect("open fst file");
    println!(
        "version: {}, date: {}",
        fst.version_string(),
        fst.date_string().trim()
    );
    println!(
        "start: {}, end: {}, timescale: 10^{}, timezero: {}",
        fst.start_time(),
        fst.end_time(),
        fst.timescale(),
        fst.timezero(),
    );
    println!(
        "value_change_section_count: {}, current_scope_len: {}, max_handle: {}",
        fst.value_change_section_count(),
        fst.current_scope_len(),
        fst.max_handle(),
    );

    println!("\nhier:");
    for hier in fst.hier_iter() {
        println!("{:?}", hier);
    }

    println!("\nblocks:");
    fst.set_limit_time_range(0..240);
    fst.iter_blocks(|time, handle, val| {
        println!("block: {time} {handle} {val:?}");
    });
}
