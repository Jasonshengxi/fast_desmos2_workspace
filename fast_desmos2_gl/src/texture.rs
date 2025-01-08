use crate::{enum_transmutable_u32, has_handle, transmutable_u32, GlErrorGuard};
use bon::Builder;
use gl::types::*;
use glam::{IVec2, IVec3, Vec4};

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    R8 = gl::R8,
    R8Snorm = gl::R8_SNORM,
    R16 = gl::R16,
    R16Snorm = gl::R16_SNORM,
    Rg8 = gl::RG8,
    Rg8Snorm = gl::RG8_SNORM,
    Rg16 = gl::RG16,
    Rg16Snorm = gl::RG16_SNORM,
    Rg3B2 = gl::R3_G3_B2,
    Rgb4 = gl::RGB4,
    Rgb5 = gl::RGB5,
    Rgb8 = gl::RGB8,
    Rgb8Snorm = gl::RGB8_SNORM,
    Rgb10 = gl::RGB10,
    Rgb12 = gl::RGB12,
    Rgb16Snorm = gl::RGB16_SNORM,
    Rgba2 = gl::RGBA2,
    Rgba4 = gl::RGBA4,
    Rgb5A1 = gl::RGB5_A1,
    Rgba8 = gl::RGBA8,
    Rgba8Snorm = gl::RGBA8_SNORM,
    Rgb10A2 = gl::RGB10_A2,
    Rgb10A2UI = gl::RGB10_A2UI,
    Rgba12 = gl::RGBA12,
    Rgba16 = gl::RGBA16,
    SRgb8 = gl::SRGB8,
    SRgb8A8 = gl::SRGB8_ALPHA8,
    R16F = gl::R16F,
    Rg16F = gl::RG16F,
    Rgb16F = gl::RGB16F,
    Rgba16F = gl::RGBA16F,
    R32F = gl::R32F,
    Rg32F = gl::RG32F,
    Rgb32F = gl::RGB32F,
    Rgba32F = gl::RGBA32F,
    Rg11FB10F = gl::R11F_G11F_B10F,
    Rgb9E5 = gl::RGB9_E5,
    R8I = gl::R8I,
    R8UI = gl::R8UI,
    R16I = gl::R16I,
    R16UI = gl::R16UI,
    R32I = gl::R32I,
    R32UI = gl::R32UI,
    Rg8I = gl::RG8I,
    Rg8UI = gl::RG8UI,
    Rg16I = gl::RG16I,
    Rg16UI = gl::RG16UI,
    Rg32I = gl::RG32I,
    Rg32UI = gl::RG32UI,
    Rgb8I = gl::RGB8I,
    Rgb8UI = gl::RGB8UI,
    Rgb16I = gl::RGB16I,
    Rgb16UI = gl::RGB16UI,
    Rgb32I = gl::RGB32I,
    Rgb32UI = gl::RGB32UI,
    Rgba8I = gl::RGBA8I,
    Rgba8UI = gl::RGBA8UI,
    Rgba16I = gl::RGBA16I,
    Rgba16UI = gl::RGBA16UI,
    Rgba32I = gl::RGBA32I,
    Rgba32UI = gl::RGBA32UI,
}
transmutable_u32!(ImageFormat);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum TextureTarget {
    Texture1D = gl::TEXTURE_1D,
    Texture2D = gl::TEXTURE_2D,
    Texture3D = gl::TEXTURE_3D,
    Array1D = gl::TEXTURE_1D_ARRAY,
    Array2D = gl::TEXTURE_2D_ARRAY,
    Rectangle = gl::TEXTURE_RECTANGLE,
    CubeMap = gl::TEXTURE_CUBE_MAP,
    ArrayCubeMap = gl::TEXTURE_CUBE_MAP_ARRAY,
    TextureBuffer = gl::TEXTURE_BUFFER,
    // TODO multisample
    // Multisample2D = gl::TEXTURE_2D_MULTISAMPLE,
    // ArrayMultisample2D = gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
}
transmutable_u32!(TextureTarget);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Texture2DTarget {
    Texture2D = gl::TEXTURE_2D,
    Rectangle = gl::TEXTURE_RECTANGLE,
    CubeMap = gl::TEXTURE_CUBE_MAP,
    Array1D = gl::TEXTURE_1D_ARRAY,
}
transmutable_u32!(Texture2DTarget);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Texture3DTarget {
    Texture3D = gl::TEXTURE_3D,
    Array2D = gl::TEXTURE_2D_ARRAY,
    ArrayCubeMap = gl::TEXTURE_CUBE_MAP_ARRAY,
}
transmutable_u32!(Texture3DTarget);

#[derive(Debug, Clone, Copy)]
pub enum TextureParams {
    Texture1D {
        params: Texture1DParams,
    },
    Texture2D {
        target: Texture2DTarget,
        params: Texture2DParams,
    },
    Texture3D {
        target: Texture3DTarget,
        params: Texture3DParams,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Texture1DParams {
    levels: i32,
    format: ImageFormat,
    width: i32,
}

impl Texture1DParams {
    fn tex_storage(&self) {
        unsafe {
            gl::TexStorage1D(
                TextureTarget::Texture1D.to_u32(),
                self.levels,
                self.format.to_u32(),
                self.width,
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Texture2DParams {
    pub levels: i32,
    pub format: ImageFormat,
    pub dim: IVec2,
}

impl Texture2DParams {
    fn tex_storage(&self, target: Texture2DTarget) {
        unsafe {
            gl::TexStorage2D(
                target.to_u32(),
                self.levels,
                self.format.to_u32(),
                self.dim.x,
                self.dim.y,
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Texture3DParams {
    pub levels: i32,
    pub format: ImageFormat,
    pub dim: IVec3,
}

impl Texture3DParams {
    fn tex_storage(&self, target: Texture3DTarget) {
        unsafe {
            gl::TexStorage3D(
                target.to_u32(),
                self.levels,
                self.format.to_u32(),
                self.dim.x,
                self.dim.y,
                self.dim.z,
            )
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Red = gl::RED,
    Rg = gl::RG,
    Rgb = gl::RGB,
    Bgr = gl::BGR,
    Rgba = gl::RGBA,
    Bgra = gl::BGRA,
    Depth = gl::DEPTH_COMPONENT,
    Stencil = gl::STENCIL_INDEX,
}
transmutable_u32!(PixelFormat);

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum PixelKind {
    U8 = gl::UNSIGNED_BYTE,
    I8 = gl::BYTE,
    U16 = gl::UNSIGNED_SHORT,
    I16 = gl::SHORT,
    U32 = gl::UNSIGNED_INT,
    I32 = gl::INT,
    F32 = gl::FLOAT,
    U8_3_3_2 = gl::UNSIGNED_BYTE_3_3_2,
    U8R_2_3_3 = gl::UNSIGNED_BYTE_2_3_3_REV,
    U16_5_6_5 = gl::UNSIGNED_SHORT_5_6_5,
    U16R_5_6_5 = gl::UNSIGNED_SHORT_5_6_5_REV,
    U16_4_4_4_4 = gl::UNSIGNED_SHORT_4_4_4_4,
    U16R_4_4_4_4 = gl::UNSIGNED_SHORT_4_4_4_4_REV,
    U16_5_5_5_1 = gl::UNSIGNED_SHORT_5_5_5_1,
    U16R_1_5_5_5 = gl::UNSIGNED_SHORT_1_5_5_5_REV,
    U32_8_8_8_8 = gl::UNSIGNED_INT_8_8_8_8,
    U32R_8_8_8_8 = gl::UNSIGNED_INT_8_8_8_8_REV,
    U32_10_10_10_2 = gl::UNSIGNED_INT_10_10_10_2,
    U32R_2_10_10_10 = gl::UNSIGNED_INT_2_10_10_10_REV,
}
transmutable_u32!(PixelKind);

enum_transmutable_u32! {
    pub enum TextureParameter {
        DepthStencilMode = gl::DEPTH_STENCIL_TEXTURE_MODE,
        BaseLevel = gl::TEXTURE_BASE_LEVEL,
        CompareFunc = gl::TEXTURE_COMPARE_FUNC,
        CompareMode = gl::TEXTURE_COMPARE_MODE,
        LodBias = gl::TEXTURE_LOD_BIAS,
        MinFilter = gl::TEXTURE_MIN_FILTER,
        MagFilter = gl::TEXTURE_MAG_FILTER,
        MinLod = gl::TEXTURE_MIN_LOD,
        MaxLod = gl::TEXTURE_MAX_LOD,
        MaxLevel = gl::TEXTURE_MAX_LEVEL,
        SwizzleR = gl::TEXTURE_SWIZZLE_R,
        SwizzleG = gl::TEXTURE_SWIZZLE_G,
        SwizzleB = gl::TEXTURE_SWIZZLE_B,
        SwizzleA = gl::TEXTURE_SWIZZLE_A,
        WrapS = gl::TEXTURE_WRAP_S,
        WrapT = gl::TEXTURE_WRAP_T,
        WrapR = gl::TEXTURE_WRAP_R,

        BorderColor = gl::TEXTURE_BORDER_COLOR,
        SwizzleRgba = gl::TEXTURE_SWIZZLE_RGBA,
    }
}

enum_transmutable_u32! {
    pub enum TextureCompareFunc {
        LessOrEqual = gl::LEQUAL,
        MoreOrEqual = gl::GEQUAL,
        Less = gl::LESS,
        More = gl::GREATER,
        Equal = gl::EQUAL,
        NotEqual = gl::NOTEQUAL,
        Always = gl::ALWAYS,
        Never = gl::NEVER,
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum TextureDepthStencilMode {
    #[default]
    Depth,
    Stencil,
}

impl TextureDepthStencilMode {
    pub const fn to_i32(self) -> i32 {
        match self {
            TextureDepthStencilMode::Depth => gl::DEPTH_COMPONENT as i32,
            TextureDepthStencilMode::Stencil => gl::STENCIL_INDEX as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FilterFunction {
    Nearest,
    #[default]
    Linear,
}

impl FilterFunction {
    pub const fn to_i32(self) -> i32 {
        match self {
            FilterFunction::Nearest => gl::NEAREST as i32,
            FilterFunction::Linear => gl::LINEAR as i32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MinFilterFunction {
    Simple(FilterFunction),
    Mipmap {
        mipmap: FilterFunction,
        pixel: FilterFunction,
    },
}

impl Default for MinFilterFunction {
    fn default() -> Self {
        Self::Mipmap {
            mipmap: FilterFunction::Linear,
            pixel: FilterFunction::Nearest,
        }
    }
}

impl MinFilterFunction {
    pub const fn to_i32(self) -> i32 {
        use FilterFunction as FF;
        match self {
            MinFilterFunction::Simple(func) => func.to_i32(),
            MinFilterFunction::Mipmap { mipmap, pixel } => match (pixel, mipmap) {
                (FF::Nearest, FF::Nearest) => gl::NEAREST_MIPMAP_NEAREST as i32,
                (FF::Nearest, FF::Linear) => gl::NEAREST_MIPMAP_LINEAR as i32,
                (FF::Linear, FF::Nearest) => gl::LINEAR_MIPMAP_NEAREST as i32,
                (FF::Linear, FF::Linear) => gl::LINEAR_MIPMAP_LINEAR as i32,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SwizzleComponent {
    Red,
    Green,
    Blue,
    Alpha,
    Zero,
    One,
}

impl SwizzleComponent {
    pub const fn to_i32(self) -> i32 {
        match self {
            SwizzleComponent::Red => gl::RED as i32,
            SwizzleComponent::Green => gl::GREEN as i32,
            SwizzleComponent::Blue => gl::BLUE as i32,
            SwizzleComponent::Alpha => gl::ALPHA as i32,
            SwizzleComponent::Zero => gl::ZERO as i32,
            SwizzleComponent::One => gl::ONE as i32,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum WrapParameter {
    ClampToEdge,
    ClampToBorder,
    MirroredRepeat,
    #[default]
    Repeat,
    MirrorClampToEdge,
}

impl WrapParameter {
    pub const fn to_i32(self) -> i32 {
        match self {
            WrapParameter::ClampToEdge => gl::CLAMP_TO_EDGE as i32,
            WrapParameter::ClampToBorder => gl::CLAMP_TO_BORDER as i32,
            WrapParameter::MirroredRepeat => gl::MIRRORED_REPEAT as i32,
            WrapParameter::Repeat => gl::REPEAT as i32,
            WrapParameter::MirrorClampToEdge => gl::MIRROR_CLAMP_TO_EDGE as i32,
        }
    }
}

#[derive(Builder)]
pub struct TextureParameters {
    #[builder(default)]
    depth_stencil_mode: TextureDepthStencilMode,
    #[builder(default)]
    base_level: i32,
    #[builder(default)]
    border_color: Vec4,
    compare: Option<TextureCompareFunc>,
    #[builder(default)]
    min_filter: MinFilterFunction,
    #[builder(default)]
    mag_filter: FilterFunction,

    #[builder(default = [
        SwizzleComponent::Red,
        SwizzleComponent::Green,
        SwizzleComponent::Blue,
        SwizzleComponent::Alpha
    ])]
    swizzle_rgba: [SwizzleComponent; 4],
    #[builder(default)]
    wrap_s: WrapParameter,
    #[builder(default)]
    wrap_t: WrapParameter,
    #[builder(default)]
    wrap_r: WrapParameter,
}

impl Default for TextureParameters {
    fn default() -> Self {
        Self {
            depth_stencil_mode: Default::default(),
            base_level: Default::default(),
            border_color: Default::default(),
            compare: Default::default(),
            min_filter: Default::default(),
            mag_filter: Default::default(),
            swizzle_rgba: [
                SwizzleComponent::Red,
                SwizzleComponent::Green,
                SwizzleComponent::Blue,
                SwizzleComponent::Alpha,
            ],
            wrap_s: Default::default(),
            wrap_t: Default::default(),
            wrap_r: Default::default(),
        }
    }
}

impl TextureParameters {
    pub fn apply(&self, texture: u32) {
        unsafe {
            let i_param = |param: TextureParameter, value| {
                gl::TextureParameteri(texture, param.to_u32(), value)
            };

            i_param(
                TextureParameter::DepthStencilMode,
                self.depth_stencil_mode.to_i32(),
            );

            i_param(TextureParameter::BaseLevel, self.base_level);

            gl::TextureParameterfv(
                texture,
                TextureParameter::BorderColor.to_u32(),
                self.border_color.to_array().as_ptr(),
            );

            i_param(TextureParameter::MinFilter, self.min_filter.to_i32());

            i_param(TextureParameter::MagFilter, self.mag_filter.to_i32());

            gl::TextureParameteriv(
                texture,
                TextureParameter::SwizzleRgba.to_u32(),
                self.swizzle_rgba
                    .map(|component| component.to_i32())
                    .as_ptr(),
            );

            i_param(TextureParameter::WrapS, self.wrap_s.to_i32());
            i_param(TextureParameter::WrapT, self.wrap_t.to_i32());
            i_param(TextureParameter::WrapR, self.wrap_r.to_i32());

            match self.compare {
                None => {
                    gl::TextureParameteri(
                        texture,
                        TextureParameter::CompareMode.to_u32(),
                        gl::NONE as i32,
                    );
                }
                Some(func) => {
                    gl::TextureParameteri(
                        texture,
                        TextureParameter::CompareMode.to_u32(),
                        gl::COMPARE_REF_TO_TEXTURE as i32,
                    );

                    gl::TextureParameteri(
                        texture,
                        TextureParameter::CompareFunc.to_u32(),
                        func.to_u32() as i32,
                    )
                }
            }

            GlErrorGuard::clear_existing(Some("setting texture parameters"));
        }
    }
}

enum_transmutable_u32! {
    pub enum TextureAccess {
        ReadOnly = gl::READ_ONLY,
        WriteOnly = gl::WRITE_ONLY,
        ReadWrite = gl::READ_WRITE,
    }
}

#[derive(Debug)]
pub struct Texture2D {
    handle: u32,
    target: Texture2DTarget,
    params: Texture2DParams,
}
has_handle!(Texture2D);

unsafe fn new_handle() -> u32 {
    let mut handle = 0;
    unsafe { gl::GenTextures(1, &mut handle) };
    assert_ne!(handle, 0, "GenTextures failed");
    handle
}

impl Texture2D {
    pub fn new(
        target: Texture2DTarget,
        params: TextureParameters,
        create_params: Texture2DParams,
    ) -> Self {
        let handle = unsafe { new_handle() };
        unsafe { gl::BindTexture(target.to_u32(), handle) };
        create_params.tex_storage(target);
        params.apply(handle);
        Self {
            handle,
            target,
            params: create_params,
        }
    }

    pub fn bind_to(&self, unit: u32) {
        unsafe { gl::BindTextureUnit(unit, self.handle) };
    }

    pub fn bind_to_image(&self, unit: u32, level: i32, access: TextureAccess) {
        unsafe {
            gl::BindImageTexture(
                unit,
                self.handle,
                level,
                gl::FALSE,
                0,
                access.to_u32(),
                self.params.format.to_u32(),
            )
        };
    }

    pub fn store_data<T>(
        &self,
        level: i32,
        offset: IVec2,
        size: IVec2,
        format: PixelFormat,
        kind: PixelKind,
        pixels: &[T],
    ) {
        unsafe {
            gl::TextureSubImage2D(
                self.handle,
                level,
                offset.x,
                offset.y,
                size.x,
                size.y,
                format.to_u32(),
                kind.to_u32(),
                pixels.as_ptr().cast(),
            )
        }
    }
}

#[derive(Debug)]
pub struct Texture3D {
    handle: u32,
    target: Texture3DTarget,
    params: Texture3DParams,
}
has_handle!(Texture3D);

impl Texture3D {
    pub fn new(
        target: Texture3DTarget,
        params: TextureParameters,
        create_params: Texture3DParams,
    ) -> Self {
        let handle = unsafe { new_handle() };
        unsafe { gl::BindTexture(target.to_u32(), handle) };
        create_params.tex_storage(target);
        params.apply(handle);
        Self {
            handle,
            target,
            params: create_params,
        }
    }

    pub fn bind_to(&self, unit: u32) {
        unsafe { gl::BindTextureUnit(unit, self.handle) };
    }

    pub fn bind_to_image(&self, unit: u32, level: i32, access: TextureAccess) {
        unsafe {
            gl::BindImageTexture(
                unit,
                self.handle,
                level,
                gl::TRUE,
                0, // this is ignored
                access.to_u32(),
                self.params.format.to_u32(),
            )
        };
    }

    pub fn store_data<T>(
        &self,
        level: i32,
        offset: IVec3,
        size: IVec3,
        format: PixelFormat,
        kind: PixelKind,
        pixels: &[T],
    ) {
        unsafe {
            gl::TextureSubImage3D(
                self.handle,
                level,
                offset.x,
                offset.y,
                offset.z,
                size.x,
                size.y,
                size.z,
                format.to_u32(),
                kind.to_u32(),
                pixels.as_ptr().cast(),
            )
        }
    }
}
