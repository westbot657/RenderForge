use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::ops::{AddAssign, MulAssign};
use std::rc::Rc;
use delegate::delegate;
use gl::types::{GLboolean, GLenum, GLint, GLsizei, GLuint};
use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

#[derive(Debug, Clone, PartialEq)]
pub struct MatrixStack {
    stack: Vec<Mat4>,
    current: Mat4,
}

impl MatrixStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Mat4::default(),
        }
    }

    pub fn push(&mut self) {
        self.stack.push(self.current)
    }

    /// panics if the stack is empty, make sure you match up every push() with a pop()
    pub fn pop(&mut self) {
        self.current = self.stack.pop().expect("MatrixStack underflow")
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.current *= Mat4::from_translation(translation)
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.current *= Mat4::from_scale(scale)
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.current *= Mat4::from_quat(rotation)
    }

    pub fn get_transform(&self) -> &Mat4 {
        &self.current
    }
}

impl MulAssign<Mat4> for MatrixStack {
    fn mul_assign(&mut self, rhs: Mat4) {
        self.current *= rhs;
    }
}

impl AddAssign<Vec3> for MatrixStack {
    /// Translates the MatrixStack
    fn add_assign(&mut self, rhs: Vec3) {
        self.translate(rhs);
    }
}

impl MulAssign<Vec3> for MatrixStack {
    /// Scales the MatrixStack
    fn mul_assign(&mut self, rhs: Vec3) {
        self.scale(rhs);
    }
}

impl MulAssign<Quat> for MatrixStack {
    /// Rotates the MatrixStack
    fn mul_assign(&mut self, rhs: Quat) {
        self.rotate(rhs);
    }
}


pub trait ColorLike {
    fn to_color(&self) -> Color;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self {
            r, g, b,
            a: 1.0
        }
    }

    pub fn from_argb(argb: u32) -> Self {
        let a = (argb >> 24) as f32 / 255.;
        let r = ((argb >> 16) & 0xFF) as f32 / 255.;
        let g = ((argb >> 8) & 0xFF) as f32 / 255.;
        let b = (argb & 0xFF) as f32 / 255.;
        Self { r, g, b, a }
    }

    pub fn to_tuple(&self) -> (f32, f32, f32, f32) {
        (self.r, self.g, self.b, self.a)
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a] 
    }

    pub fn to_argb(&self) -> u32 {
        let r = (self.r * 255.) as u32 & 0xFFu32;
        let g = (self.g * 255.) as u32 & 0xFFu32;
        let b = (self.b * 255.) as u32 & 0xFFu32;
        let a = (self.a * 255.) as u32 & 0xFFu32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

}


#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthFunc {
    Never,
    Less,
    Equal,
    LEqual,
    Greater,
    GEqual,
    NotEqual,
    Always,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CullFace {
    Back,
    Front,
    FrontAndBack,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Winding {
    CW,
    CCW
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SrcRgb {
    Factor(BlendFactor),
    SrcAlphaSaturate,
}

/// Aliases for clarity — these all use the shared BlendFactor
pub type DstRgb = BlendFactor;
pub type SrcAlpha = BlendFactor;
pub type DstAlpha = BlendFactor;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RgbEquation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AlphaEquation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StencilFunc {
    Never,
    Less,
    Lequal,
    Greater,
    Gequal,
    Equal,
    NotEqual,
    Always,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Incr,
    IncrWrap,
    Decr,
    DecrWrap,
    Invert,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StencilFace {
    Front,
    Back,
    FrontAndBack,
}


impl DepthFunc {
    pub fn to_gl(&self) -> GLenum {
        match self {
            DepthFunc::Never => gl::NEVER,
            DepthFunc::Less => gl::LESS,
            DepthFunc::Equal => gl::EQUAL,
            DepthFunc::LEqual => gl::LEQUAL,
            DepthFunc::Greater => gl::GREATER,
            DepthFunc::GEqual => gl::GEQUAL,
            DepthFunc::NotEqual => gl::NOTEQUAL,
            DepthFunc::Always => gl::ALWAYS,
        }
    }
}

impl CullFace {
    pub fn to_gl(&self) -> GLenum {
        match self {
            CullFace::Back => gl::BACK,
            CullFace::Front => gl::FRONT,
            CullFace::FrontAndBack => gl::FRONT_AND_BACK,
        }
    }
}

impl Winding {
    pub fn to_gl(&self) -> GLenum {
        match self {
            Winding::CW => gl::CW,
            Winding::CCW => gl::CCW,
        }
    }
}

impl BlendFactor {
    pub fn to_gl(&self) -> GLenum {
        match self {
            BlendFactor::Zero => gl::ZERO,
            BlendFactor::One => gl::ONE,
            BlendFactor::SrcColor => gl::SRC_COLOR,
            BlendFactor::OneMinusSrcColor => gl::ONE_MINUS_SRC_COLOR,
            BlendFactor::DstColor => gl::DST_COLOR,
            BlendFactor::OneMinusDstColor => gl::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => gl::SRC_ALPHA,
            BlendFactor::OneMinusSrcAlpha => gl::ONE_MINUS_SRC_ALPHA,
            BlendFactor::DstAlpha => gl::DST_ALPHA,
            BlendFactor::OneMinusDstAlpha => gl::ONE_MINUS_DST_ALPHA,
            BlendFactor::ConstantColor => gl::CONSTANT_COLOR,
            BlendFactor::OneMinusConstantColor => gl::ONE_MINUS_CONSTANT_COLOR,
            BlendFactor::ConstantAlpha => gl::CONSTANT_ALPHA,
            BlendFactor::OneMinusConstantAlpha => gl::ONE_MINUS_CONSTANT_ALPHA,
        }
    }
}
impl SrcRgb {
    pub fn to_gl(&self) -> GLenum {
        match self {
            SrcRgb::Factor(f) => f.to_gl(),
            SrcRgb::SrcAlphaSaturate => gl::SRC_ALPHA_SATURATE,
        }
    }
}

impl RgbEquation {
    pub fn to_gl(&self) -> GLenum {
        match self {
            RgbEquation::Add => gl::FUNC_ADD,
            RgbEquation::Subtract => gl::FUNC_SUBTRACT,
            RgbEquation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
            RgbEquation::Min => gl::MIN,
            RgbEquation::Max => gl::MAX,
        }
    }
}

impl AlphaEquation {
    pub fn to_gl(&self) -> GLenum {
        match self {
            AlphaEquation::Add => gl::FUNC_ADD,
            AlphaEquation::Subtract => gl::FUNC_SUBTRACT,
            AlphaEquation::ReverseSubtract => gl::FUNC_REVERSE_SUBTRACT,
            AlphaEquation::Min => gl::MIN,
            AlphaEquation::Max => gl::MAX,
        }
    }
}

impl StencilFunc {
    pub fn to_gl(&self) -> GLenum {
        match self {
            StencilFunc::Never => gl::NEVER,
            StencilFunc::Less => gl::LESS,
            StencilFunc::Lequal => gl::LEQUAL,
            StencilFunc::Greater => gl::GREATER,
            StencilFunc::Gequal => gl::GEQUAL,
            StencilFunc::Equal => gl::EQUAL,
            StencilFunc::NotEqual => gl::NOTEQUAL,
            StencilFunc::Always => gl::ALWAYS,
        }
    }
}

impl StencilOp {
    pub fn to_gl(&self) -> GLenum {
        match self {
            StencilOp::Keep => gl::KEEP,
            StencilOp::Zero => gl::ZERO,
            StencilOp::Replace => gl::REPLACE,
            StencilOp::Incr => gl::INCR,
            StencilOp::IncrWrap => gl::INCR_WRAP,
            StencilOp::Decr => gl::DECR,
            StencilOp::DecrWrap => gl::DECR_WRAP,
            StencilOp::Invert => gl::INVERT,
        }
    }
}

impl StencilFace {
    pub fn to_gl(&self) -> GLenum {
        match self {
            StencilFace::Front => gl::FRONT,
            StencilFace::Back => gl::BACK,
            StencilFace::FrontAndBack => gl::FRONT_AND_BACK,
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct DepthState {
    pub enabled: bool,
    pub func: DepthFunc,
    pub mask: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct CullState {
    pub enabled: bool,
    pub face: CullFace,
    pub front_face: Winding,
}
#[derive(Debug, Copy, Clone)]
pub struct BlendState {
    pub enabled: bool,
    pub src_rgb: SrcRgb,
    pub src_alpha: SrcAlpha,
    pub dst_rgb: DstRgb,
    pub dst_alpha: DstAlpha,
    pub rgb_equation: RgbEquation,
    pub alpha_equation: AlphaEquation
}
#[derive(Debug, Copy, Clone)]
pub struct StencilState {
    pub enabled: bool,
    pub face: StencilFace,
    pub func: StencilFunc,
    pub reference: i32,
    pub mask: GLuint,
    pub fail_op: StencilOp,
    pub z_fail_op: StencilOp,
    pub z_pass_op: StencilOp,
}
#[derive(Debug, Copy, Clone)]
pub struct RasterState {
    pub scissor_test: bool,
    pub scissor_box: [i32; 4],
    pub viewport: [i32; 4],
}
#[derive(Debug, Clone)]
pub struct SamplerState {

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GLUniform {
    Mat4(Mat4),
    F32(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    I32(i32),
    IVec2(IVec2),
    IVec3(IVec3),
    IVec4(IVec4),
}
impl GLUniform {
    /// # Safety
    /// As long as loc is a valid uniform location, this method is safe.
    pub unsafe fn upload(&self, loc: GLint) {
        unsafe {
            match self {
                GLUniform::Mat4(m) => {
                    let mat = m.to_cols_array();
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, mat.as_ptr());
                }
                GLUniform::F32(f) => {
                    gl::Uniform1f(loc, *f);
                }
                GLUniform::Vec2(v) => {
                    gl::Uniform2f(loc, v.x, v.y);
                }
                GLUniform::Vec3(v) => {
                    gl::Uniform3f(loc, v.x, v.y, v.z);
                }
                GLUniform::Vec4(v) => {
                    gl::Uniform4f(loc, v.x, v.y, v.z, v.w);
                }
                GLUniform::I32(i) => {
                    gl::Uniform1i(loc, *i);
                }
                GLUniform::IVec2(v) => {
                    gl::Uniform2i(loc, v.x, v.y);
                }
                GLUniform::IVec3(v) => {
                    gl::Uniform3i(loc, v.x, v.y, v.z);
                }
                GLUniform::IVec4(v) => {
                    gl::Uniform4i(loc, v.x, v.y, v.z, v.w);
                }
            }
        }
    }
}

/// Not meant to be hand-constructed, relies on GlStateManager for manipulation and for GlStateSnapshot to work
#[derive(Debug, Clone)]
pub struct GlState {
    pub depth: DepthState,
    pub cull: CullState,
    pub blend: BlendState,
    pub stencil: StencilState,
    pub raster: RasterState,
    pub sampler: SamplerState,

    vao: GLuint,
    fbo: GLuint,
    program: GLuint,
    framebuffer: GLuint,

    uniforms: HashMap<GLuint, HashMap<String, GLUniform>>,

}

pub struct GlStateRef {
    state: Rc<RefCell<GlState>>
}

pub struct GlStateManager {
    state: Rc<RefCell<GlState>>
}

/// Dropping this will reset the GL state to match when it was created
pub struct GlStateSnapshot {
    save_state: GlState,
    true_state: Rc<RefCell<GlState>>
}

impl GlState {

    pub fn depth_test(&mut self, enabled: bool) {
        if self.depth.enabled != enabled {
            self.depth.enabled = enabled;
            unsafe {
                if enabled {
                    gl::Enable(gl::DEPTH_TEST);
                } else {
                    gl::Disable(gl::DEPTH_TEST);
                }
            }
        }
    }

    pub fn depth_mask(&mut self, enabled: bool) {
        if self.depth.mask != enabled {
            self.depth.mask = enabled;
            unsafe {
                gl::DepthMask(enabled as GLboolean);
            }
        }
    }

    pub fn culling(&mut self, enabled: bool) {
        if self.cull.enabled != enabled {
            self.cull.enabled = enabled;
            unsafe {
                if enabled {
                    gl::Enable(gl::CULL_FACE);
                } else {
                    gl::Disable(gl::CULL_FACE);
                }
            }
        }
    }

    pub fn cull_face(&mut self, face: CullFace) {
        if self.cull.face != face {
            self.cull.face = face;
            unsafe {
                gl::CullFace(face.to_gl())
            }
        }
    }

    pub fn front_face(&mut self, winding: Winding) {
        if self.cull.front_face != winding {
            self.cull.front_face = winding;
            unsafe {
                gl::FrontFace(winding.to_gl());
            }
        }
    }

    pub fn blending(&mut self, enabled: bool) {
        if self.blend.enabled != enabled {
            self.blend.enabled = enabled;
            unsafe {
                if enabled {
                    gl::Enable(gl::BLEND);
                } else {
                    gl::Disable(gl::BLEND);
                }
            }
        }
    }

    pub fn blend_func_both(&mut self, src: BlendFactor, dst: BlendFactor) {
        self.blend_func_separate(SrcRgb::Factor(src), src, dst, dst)
    }

    /// SrcAlpha, DstRgb, and DstAlpha are type aliases for BlendFactor. SrcRgb is a wrapper, providing one more valid state
    pub fn blend_func(&mut self, src_rgb: SrcRgb, src_alpha: SrcAlpha, dst_rgb: DstRgb, dst_alpha: DstAlpha, rgb_equation: RgbEquation, alpha_equation: AlphaEquation) {
        self.blend_func_separate(src_rgb, src_alpha, dst_rgb, dst_alpha);
        self.blend_equation(rgb_equation, alpha_equation);
    }

    pub fn blend_func_separate(&mut self, src_rgb: SrcRgb, src_alpha: SrcAlpha, dst_rgb: DstRgb, dst_alpha: DstAlpha) {
        if self.blend.src_rgb != src_rgb
            || self.blend.src_alpha != src_alpha
            || self.blend.dst_rgb != dst_rgb
            || self.blend.dst_alpha != dst_alpha
        {
            self.blend.src_rgb = src_rgb;
            self.blend.src_alpha = src_alpha;
            self.blend.dst_rgb = dst_rgb;
            self.blend.dst_alpha = dst_alpha;
            unsafe {
                gl::BlendFuncSeparate(src_rgb.to_gl(), src_alpha.to_gl(), dst_rgb.to_gl(), dst_alpha.to_gl());
            }
        }
    }

    pub fn blend_func_rgb(&mut self, src_rgb: SrcRgb, dst_rgb: DstRgb) {
        if self.blend.src_rgb != src_rgb || self.blend.dst_rgb != dst_rgb {
            self.blend.src_rgb = src_rgb;
            self.blend.dst_rgb = dst_rgb;
            unsafe {
                gl::BlendFunc(src_rgb.to_gl(), dst_rgb.to_gl());
            }
        }
    }

    pub fn blend_equation(&mut self, rgb_equation: RgbEquation, alpha_equation: AlphaEquation) {
        if self.blend.rgb_equation != rgb_equation || self.blend.alpha_equation != alpha_equation {
            self.blend.rgb_equation = rgb_equation;
            self.blend.alpha_equation = alpha_equation;
            unsafe {
                gl::BlendEquationSeparate(rgb_equation.to_gl(), alpha_equation.to_gl());
            }
        }
    }

    pub fn use_program(&mut self, program: GLuint) {
        if self.program != program {
            self.program = program;
            unsafe {
                gl::UseProgram(program);
            }
        }
    }

    pub fn bind_vao(&mut self, vao: GLuint) {
        if self.vao != vao {
            self.vao = vao;
            unsafe {
                gl::BindVertexArray(vao);
            }
        }
    }

    pub fn bind_fbo(&mut self, fbo: GLuint) {
        if self.fbo != fbo {
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            }
        }
    }

    pub fn set_uniform(&mut self, name: impl ToString, value: GLUniform) {
        let name = name.to_string();

        if !self.uniforms.contains_key(&self.program) {
            self.uniforms.insert(self.program, HashMap::new());
        }

        let uniforms = self.uniforms.get_mut(&self.program).unwrap();

        unsafe {
            if let Some(v) = uniforms.get(&name) {
                if *v != value {
                    let cstr = CString::new(name.clone()).unwrap();
                    let loc = gl::GetUniformLocation(self.program, cstr.as_ptr());
                    value.upload(loc);
                    uniforms.insert(name, value);
                }
            } else {
                uniforms.insert(name.clone(), value);
                let cstr = CString::new(name).unwrap();
                let loc = gl::GetUniformLocation(self.program, cstr.as_ptr());
                value.upload(loc);
            }
        }

    }

    pub fn bind_texture(&mut self, slot: u32, tex: GLuint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, tex);
        }
    }

    pub fn bind_sampler(&mut self, slot: u32, sampler: GLuint) {
        unsafe {
            gl::BindSampler(slot, sampler);
        }
    }

    pub fn destroy_program(&mut self, program: GLuint) {
        if self.program == program {
            self.use_program(0);
        }
        self.uniforms.remove(&program);
        unsafe {
            gl::DeleteProgram(program);
        }
    }

    pub fn destroy_vbo_vec(&mut self, vbos: Vec<GLuint>) {
        unsafe {
            gl::DeleteBuffers(vbos.len() as GLsizei, vbos.as_ptr())
        }
    }
    pub fn destroy_vbo_box_array(&mut self, vbos: Box<[GLuint]>) {
        unsafe {
            gl::DeleteBuffers(vbos.len() as GLsizei, vbos.as_ptr())
        }
    }

    pub fn destroy_vao(&mut self, vao: GLuint) {
        unsafe {
            gl::DeleteVertexArrays(1, &vao);
        }
    }

    pub fn set_state(&mut self, state: &GlState) {
        self.use_program(state.program);
        self.bind_fbo(state.fbo);
        self.bind_vao(state.vao);
        self.blend_func(state.blend.src_rgb, state.blend.src_alpha, state.blend.dst_rgb, state.blend.dst_alpha, state.blend.rgb_equation, state.blend.alpha_equation);
        self.blending(state.blend.enabled);
        self.depth_test(state.depth.enabled);
        self.depth_mask(state.depth.mask);
        self.culling(state.cull.enabled);
        self.cull_face(state.cull.face);
        // TODO: set the rest of the states
    }

    pub fn new() -> Self {
        Self {
            depth: DepthState {
                enabled: false,
                func: DepthFunc::Less,
                mask: true,
            },
            cull: CullState {
                enabled: false,
                face: CullFace::Back,
                front_face: Winding::CCW,
            },
            blend: BlendState {
                enabled: false,
                src_rgb: SrcRgb::Factor(BlendFactor::One),
                src_alpha: BlendFactor::One,
                dst_rgb: BlendFactor::Zero,
                dst_alpha: BlendFactor::Zero,
                rgb_equation: RgbEquation::Add,
                alpha_equation: AlphaEquation::Add,
            },
            stencil: StencilState {
                enabled: false,
                func: StencilFunc::Always,
                reference: 0,
                mask: !0,
                fail_op: StencilOp::Keep,
                z_fail_op: StencilOp::Keep,
                z_pass_op: StencilOp::Keep,
                face: StencilFace::FrontAndBack,
            },
            raster: RasterState {
                scissor_test: false,
                scissor_box: [0, 0, 8096, 8096],
                viewport: [0, 0, 8096, 8096],
            },
            sampler: SamplerState {

            },
            vao: 0,
            fbo: 0,
            program: 0,
            framebuffer: 0,
            uniforms: HashMap::new(),
        }
    }

}

impl Drop for GlStateSnapshot {
    fn drop(&mut self) {
        self.true_state.borrow_mut().set_state(&self.save_state)
    }
}


impl GlStateManager {

    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(GlState::new()))
        }
    }

    pub fn snapshot(&self) -> GlStateSnapshot {
        let save_state = self.state.borrow().clone();
        GlStateSnapshot {
            save_state,
            true_state: Rc::clone(&self.state)
        }
    }

    pub fn copy_state(&self) -> GlState {
        self.state.borrow().clone()
    }

    delegate! {
        to self.state.borrow_mut() {
            pub fn depth_test(&mut self, enabled: bool);
            pub fn depth_mask(&mut self, enabled: bool);
            pub fn culling(&mut self, enabled: bool);
            pub fn cull_face(&mut self, face: CullFace);
            pub fn front_face(&mut self, winding: Winding);
            pub fn blending(&mut self, enabled: bool);
            pub fn blend_func_both(&mut self, src: BlendFactor, dst: BlendFactor);
            pub fn blend_func(&mut self, src_rgb: SrcRgb, src_alpha: SrcAlpha, dst_rgb: DstRgb, dst_alpha: DstAlpha, rgb_equation: RgbEquation, alpha_equation: AlphaEquation);
            pub fn blend_func_separate(&mut self, src_rgb: SrcRgb, src_alpha: SrcAlpha, dst_rgb: DstRgb, dst_alpha: DstAlpha);
            pub fn blend_func_rgb(&mut self, src_rgb: SrcRgb, dst_rgb: DstRgb);
            pub fn blend_equation(&mut self, rgb_equation: RgbEquation, alpha_equation: AlphaEquation);
            pub fn use_program(&mut self, program: GLuint);
            pub fn bind_vao(&mut self, vao: GLuint);
            pub fn bind_fbo(&mut self, fbo: GLuint);
            pub fn set_uniform(&mut self, name: impl ToString, value: GLUniform);
            pub fn bind_texture(&mut self, slot: u32, tex: GLuint);
            pub fn bind_sampler(&mut self, slot: u32, sampler: GLuint);
            pub fn destroy_program(&mut self, program: GLuint);
            pub fn destroy_vbo_vec(&mut self, vbos: Vec<GLuint>);
            pub fn destroy_vbo_box_array(&mut self, vbos: Box<[GLuint]>);
            pub fn destroy_vao(&mut self, vao: GLuint);
            pub fn set_state(&mut self, state: &GlState);
        }
    }

}

pub trait GLUploader {
    fn upload_gl(&self, buffer: &mut Vec<f32>);
}

impl GLUploader for Mat4 {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.append(&mut Vec::from(self.to_cols_array()))
    }
}

impl GLUploader for Vec4 {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.append(&mut vec![self.x, self.y, self.z, self.w])
    }
}

impl GLUploader for Vec3 {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.append(&mut vec![self.x, self.y, self.z])
    }
}

impl GLUploader for Vec2 {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.append(&mut vec![self.x, self.y])
    }
}

impl GLUploader for f32 {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.push(*self);
    }
}

impl GLUploader for Color {
    fn upload_gl(&self, buffer: &mut Vec<f32>) {
        buffer.append(&mut vec![self.r, self.g, self.b, self.a]);
    }
}


impl Default for GlState {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MatrixStack {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GlStateManager {
    fn default() -> Self {
        Self::new()
    }
}





