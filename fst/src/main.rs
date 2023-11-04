#![allow(dead_code)]

use nom::*;

const FST_HDR_SIM_VERSION_SIZE: usize = 1;
const FST_HDR_DATE_SIZE: usize = 1;
const FST_DOUBLE_ENDTEST: f64 = 2.7182818284590452354;
#[derive(Clone, Debug)]
struct FstHeader {
    start_time: u64,
    end_time: u64,
    double_endian_match: bool,
    mem_used_by_writer: u64,
    scope_count: u64,
    var_count: u64,
    max_handle: u64,
    num_alias: u64, // var_count - max_handle
    vc_section_count: u64,
    timescale: i8,
    version: [u8; FST_HDR_SIM_VERSION_SIZE],
    date: [u8; FST_HDR_DATE_SIZE],
    file_type: u8,
    time_zero: u64,
}

type FstHandle = u32;

#[derive(Clone, Debug)]
struct FstHierScope {
    typ: FstScopeType,
    name: String,
    component: String,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstScopeType {
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

#[derive(Clone, Debug)]
struct FstHierVar {
    typ: FstVarType,
    direction: FstVarDir,
    svt_workspace: u8,
    sdt_workspace: u8,
    sxt_workspace: u8,
    name: String,
    length: u32,
    handle: FstHandle,
    is_alias: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum FstVarType {
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
    VcdTriand = 10,
    VcdTrior = 11,
    VcdTrireg = 12,
    VcdTri0 = 13,
    VcdTri1 = 14,
    VcdWand = 15,
    VcdWire = 16,
    VcdWor = 17,
    VcdPort = 18,
    VcdSparray = 19, /* used to define the rownum (index) port for a sparse array */
    VcdRealtime = 20,

    GenString = 21, /* generic string type   (max len is defined dynamically via fstWriterEmitVariableLengthValueChange) */

    SvBit = 22,
    SvLogic = 23,
    SvInt = 24,       /* declare as size = 32 */
    SvShortint = 25,  /* declare as size = 16 */
    SvLongint = 26,   /* declare as size = 64 */
    SvByte = 27,      /* declare as size = 8  */
    SvEnum = 28,      /* declare as appropriate type range */
    SvShortreal = 29, /* declare and emit same as VcdReal (needs to be emitted as double, not a float) */
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstVarDir {
    Implicit = 0,
    Input = 1,
    Output = 2,
    Inout = 3,
    Buffer = 4,
    Linkage = 5,
}

#[derive(Clone, Debug)]
struct FstHierAttr {
    typ: FstAttrType,
    name: String,
    arg: u64,
    arg_from_name: u64,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstAttrType {
    Misc(FstMiscType) = 0, /* self-contained: does not need matching FST_HT_ATTREND */
    Array(FstArrayType) = 1,
    Enum(FstEnumValueType) = 2,
    Pack(FstPackType) = 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstMiscType {
    Comment = 0,     /* use fstWriterSetComment() to emit */
    Envvar = 1,      /* use fstWriterSetEnvVar() to emit */
    Supvar = 2,      /* use fstWriterCreateVar2() to emit */
    Pathname = 3,    /* reserved for fstWriterSetSourceStem() string -> number management */
    Sourcestem = 4,  /* use fstWriterSetSourceStem() to emit */
    Sourceistem = 5, /* use fstWriterSetSourceInstantiationStem() to emit */
    Valuelist = 6,   /* use fstWriterSetValueList() to emit, followed by fstWriterCreateVar*() */
    Enumtable = 7,   /* use fstWriterCreateEnumTable() and fstWriterEmitEnumTableRef() to emit */
    Unknown = 8,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstArrayType {
    None = 0,
    Unpacked = 1,
    Packed = 2,
    Sparse = 3,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstEnumValueType {
    SvInteger = 0,
    SvBit = 1,
    SvLogic = 2,
    SvInt = 3,
    SvShortint = 4,
    SvLongint = 5,
    SvByte = 6,
    SvUnsignedInteger = 7,
    SvUnsignedBit = 8,
    SvUnsignedLogic = 9,
    SvUnsignedInt = 10,
    SvUnsignedShortint = 11,
    SvUnsignedLongint = 12,
    SvUnsignedByte = 13,

    Reg = 14,
    Time = 15,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum FstPackType {
    None = 0,
    Unpacked = 1,
    Packed = 2,
    TaggedPacked = 3,
}

#[derive(Clone, Debug)]
#[repr(u8)]
enum FstHier {
    Scope(FstHierScope),
    Upscope,
    Var(FstHierVar),
    AttrBegin(FstHierAttr),
    AttrEnd,
    TreeBegin,
    TreeEnd,
}

#[derive(Clone, Debug)]
#[repr(u8)]
enum FstBlock {
    Header(FstHeader) = 0,
    VCData = 1,
    Blackout = 2,
    Geom = 3,
    Hier(FstHier) = 4,
    VCDataDynAlias = 5,
    HierLZ4 = 6,
    HierLZ4Duo = 7,
    VCDataDynAlias2 = 8,
    ZWrapper = 254,
}

fn parse_u64(input: &[u8]) -> IResult<&[u8], u64> {
    number::streaming::be_u64(input)
}

fn parse_vu32(input: &[u8]) -> IResult<&[u8], u32> {
    todo!()
}

fn fst_header(input: &[u8]) -> IResult<&[u8], FstHeader> {
    let parse_f64 = number::streaming::be_f64;
    let (input, start_time) = parse_u64(input)?;
    let (input, end_time) = parse_u64(input)?;
    let (input, dcheck) = parse_f64(input)?;
    let double_endian_match = dcheck == FST_DOUBLE_ENDTEST;
    if dcheck != FST_DOUBLE_ENDTEST {
        if f64::from_le_bytes(dcheck.to_be_bytes()) != FST_DOUBLE_ENDTEST {
            return combinator::fail(input);
        }
    }
    let (input, mem_used_by_writer) = parse_u64(input)?;
    let (input, scope_count) = parse_u64(input)?;
    let (input, var_count) = parse_u64(input)?;
    let (input, max_handle) = parse_u64(input)?;
    let num_alias = var_count - max_handle;
    let (input, vc_section_count) = parse_u64(input)?;
    let (input, timescale) = number::streaming::i8(input)?;
    let (input, version) = bytes::streaming::take(FST_HDR_SIM_VERSION_SIZE)(input)?;
    let version: [u8; FST_HDR_SIM_VERSION_SIZE] = version.try_into().unwrap();
    let (input, date) = bytes::streaming::take(FST_HDR_DATE_SIZE)(input)?;
    let date: [u8; FST_HDR_DATE_SIZE] = date.try_into().unwrap();
    let (input, file_type) = number::streaming::u8(input)?;
    let (input, time_zero) = parse_u64(input)?;
    Ok((
        input,
        FstHeader {
            start_time,
            end_time,
            double_endian_match,
            mem_used_by_writer,
            scope_count,
            var_count,
            max_handle,
            num_alias,
            vc_section_count,
            timescale,
            version,
            date,
            file_type,
            time_zero,
        },
    ))
}

fn fst_hier_scope(input: &[u8]) -> IResult<&[u8], FstHierScope> {
    let (input, typ) = bytes::streaming::take(1u8)(input)?;
    let typ = match typ[0] {
        0 => FstScopeType::VcdModule,
        1 => FstScopeType::VcdTask,
        2 => FstScopeType::VcdFunction,
        3 => FstScopeType::VcdBegin,
        4 => FstScopeType::VcdFork,
        5 => FstScopeType::VcdGenerate,
        6 => FstScopeType::VcdStruct,
        7 => FstScopeType::VcdUnion,
        8 => FstScopeType::VcdClass,
        9 => FstScopeType::VcdInterface,
        10 => FstScopeType::VcdPackage,
        11 => FstScopeType::VcdProgram,

        12 => FstScopeType::VhdlArchitecture,
        13 => FstScopeType::VhdlProcedure,
        14 => FstScopeType::VhdlFunction,
        15 => FstScopeType::VhdlRecord,
        16 => FstScopeType::VhdlProcess,
        17 => FstScopeType::VhdlBlock,
        18 => FstScopeType::VhdlForGenerate,
        19 => FstScopeType::VhdlIfGenerate,
        20 => FstScopeType::VhdlGenerate,
        21 => FstScopeType::VhdlPackage,
        _ => panic!("bad scope type"),
    };
    todo!()
}

fn fst_hier_var(typ: u8) -> impl Fn(&[u8]) -> IResult<&[u8], FstHierVar> {
    move |input| {
        let typ = match typ {
            0 => FstVarType::VcdEvent,
            1 => FstVarType::VcdInteger,
            2 => FstVarType::VcdParameter,
            3 => FstVarType::VcdReal,
            4 => FstVarType::VcdRealParameter,
            5 => FstVarType::VcdReg,
            6 => FstVarType::VcdSupply0,
            7 => FstVarType::VcdSupply1,
            8 => FstVarType::VcdTime,
            9 => FstVarType::VcdTri,
            10 => FstVarType::VcdTriand,
            11 => FstVarType::VcdTrior,
            12 => FstVarType::VcdTrireg,
            13 => FstVarType::VcdTri0,
            14 => FstVarType::VcdTri1,
            15 => FstVarType::VcdWand,
            16 => FstVarType::VcdWire,
            17 => FstVarType::VcdWor,
            18 => FstVarType::VcdPort,
            19 => FstVarType::VcdSparray, /* used to define the rownum (index) port for a sparse array */
            20 => FstVarType::VcdRealtime,

            21 => FstVarType::GenString, /* generic string type   (max len is defined dynamically via fstWriterEmitVariableLengthValueChange) */

            22 => FstVarType::SvBit,
            23 => FstVarType::SvLogic,
            24 => FstVarType::SvInt,       /* declare as size = 32 */
            25 => FstVarType::SvShortint,  /* declare as size = 16 */
            26 => FstVarType::SvLongint,   /* declare as size = 64 */
            27 => FstVarType::SvByte,      /* declare as size = 8  */
            28 => FstVarType::SvEnum,      /* declare as appropriate type range */
            29 => FstVarType::SvShortreal, /* declare and emit same as VcdReal (needs to be emitted as double, not a float) */
            _ => panic!("bad var type"),
        };
        let (input, dir) = bytes::streaming::take(1u8)(input)?;
        let direction = match dir[0] {
            0 => FstVarDir::Implicit,
            1 => FstVarDir::Input,
            2 => FstVarDir::Output,
            3 => FstVarDir::Inout,
            4 => FstVarDir::Buffer,
            5 => FstVarDir::Linkage,
            _ => panic!("bad var direction"),
        };
        let (input, mut name) = bytes::streaming::take_till(|b| b == 0)(input)?;
        if name.ends_with(&[0u8][..]) {
            name = &name[..name.len() - 1];
        }
        let name = std::str::from_utf8(&name)
            .expect("invalid utf8 varname")
            .to_string();
        let (input, mut length) = parse_vu32(input)?;
        if typ == FstVarType::VcdPort {
            length -= 2; /* removal of delimiting spaces */
            length /= 3; /* port -> signal size adjust */
        }
        let (input, alias) = parse_vu32(input)?;
        Ok((
            input,
            FstHierVar {
                typ,
                direction,
                svt_workspace: 0,
                sdt_workspace: 0,
                sxt_workspace: 0,
                name,
                length,
                handle: alias,
                is_alias: alias != 0,
            },
        ))
    }
}

fn fst_hier_attr(input: &[u8]) -> IResult<&[u8], FstHierAttr> {
    todo!()
}

fn fst_hier(input: &[u8]) -> IResult<&[u8], FstHier> {
    let (input, typ) = bytes::streaming::take(1u8)(input)?;
    match typ[0] {
        254 => combinator::map(fst_hier_scope, FstHier::Scope)(input),
        255 => Ok((input, FstHier::Upscope)),
        252 => combinator::map(fst_hier_attr, FstHier::AttrBegin)(input),
        253 => Ok((input, FstHier::AttrEnd)),
        n if n >= (FstVarType::VcdEvent as u8) && n <= (FstVarType::SvShortreal as u8) => {
            combinator::map(fst_hier_var(n), FstHier::Var)(input)
        }
        _ => panic!("invalid type"),
    }
}

fn fst_block(input: &[u8]) -> IResult<&[u8], FstBlock> {
    let (input, sectype) = bytes::streaming::take(1u8)(input)?;
    let (input, _seclen) = parse_u64(input)?;
    match sectype[0] {
        0 => combinator::map(fst_header, FstBlock::Header)(input),
        1 => Ok((input, FstBlock::VCData)),
        2 => Ok((input, FstBlock::Blackout)),
        3 => Ok((input, FstBlock::Geom)),
        4 => todo!("FST_BL_HIER"),
        5 => Ok((input, FstBlock::VCDataDynAlias)),
        6 => todo!("FST_BL_HIER_LZ4"),
        7 => todo!("FST_BL_HIER_LZ4DUO"),
        8 => Ok((input, FstBlock::VCDataDynAlias2)),

        254 => todo!("FST_BL_ZWRAPPER"), /* indicates that whole trace is gz wrapped */
        255 => todo!("FST_BL_SKIP"),     /* used while block is being written */
        _ => panic!("bad sectype"),
    }
}

fn main() {
    println!(
        "Hello, world! {:x}",
        u64::from_ne_bytes(FST_DOUBLE_ENDTEST.to_ne_bytes())
    );
}
