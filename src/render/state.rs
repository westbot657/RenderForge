use std::ops::MulAssign;
use glam::{Mat4, Quat, Vec3};
use glow::HasContext;

#[derive(Clone)]
pub struct MatrixStack {
    pub stack: Vec<Mat4>,
    pub current: Mat4,
}

impl MatrixStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: Mat4::IDENTITY,
        }
    }

    pub fn push(&mut self) {
        self.stack.push(self.current);
    }

    pub fn pop(&mut self) {
        self.current = self.stack.pop().unwrap_or(Mat4::IDENTITY);
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.current *= Mat4::from_translation(translation);
    }

    pub fn scale(&mut self, scale: Vec3) {
        self.current *= Mat4::from_scale(scale);
    }

    pub fn rotate(&mut self, rotation: Quat) {
        self.current *= Mat4::from_quat(rotation);
    }
}

impl MulAssign<Mat4> for MatrixStack {
    fn mul_assign(&mut self, rhs: Mat4) {
        self.current *= rhs;
    }
}

impl Default for MatrixStack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DepthState {
    pub enabled: bool,
    /// `glow::LESS`, `glow::EQUAL`, etc.
    pub func: u32,
    /// Depth write enable.
    pub mask: bool,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            enabled: false,
            func: glow::LESS,
            mask: true,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CullState {
    pub enabled: bool,
    /// `glow::BACK`, `glow::FRONT`, `glow::FRONT_AND_BACK`.
    pub face: u32,
    /// `glow::CCW` or `glow::CW`.
    pub front_face: u32,
}

impl Default for CullState {
    fn default() -> Self {
        Self {
            enabled: false,
            face: glow::BACK,
            front_face: glow::CCW,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlendState {
    pub enabled: bool,
    pub src_rgb: u32,
    pub dst_rgb: u32,
    pub src_alpha: u32,
    pub dst_alpha: u32,
    pub equation_rgb: u32,
    pub equation_alpha: u32,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            enabled: false,
            src_rgb: glow::ONE,
            dst_rgb: glow::ZERO,
            src_alpha: glow::ONE,
            dst_alpha: glow::ZERO,
            equation_rgb: glow::FUNC_ADD,
            equation_alpha: glow::FUNC_ADD,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StencilState {
    pub enabled: bool,
    pub func: u32,
    pub reference: i32,
    pub mask: u32,
    pub fail_op: u32,
    pub z_fail_op: u32,
    pub z_pass_op: u32,
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            enabled: false,
            func: glow::ALWAYS,
            reference: 0,
            mask: !0,
            fail_op: glow::KEEP,
            z_fail_op: glow::KEEP,
            z_pass_op: glow::KEEP,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RasterState {
    pub scissor_test: bool,
    pub scissor_box: [i32; 4],
    pub viewport: [i32; 4],
}

impl Default for RasterState {
    fn default() -> Self {
        Self {
            scissor_test: false,
            scissor_box: [0, 0, 8096, 8096],
            viewport: [0, 0, 8096, 8096],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GlState {
    pub depth: DepthState,
    pub cull: CullState,
    pub blend: BlendState,
    pub stencil: StencilState,
    pub raster: RasterState,
}

pub struct GlStateManager {
    state: GlState,
}

impl GlStateManager {
    pub fn new() -> Self {
        Self {
            state: GlState::default(),
        }
    }

    // snapshot / restore

    pub fn snapshot(&self) -> GlState {
        self.state.clone()
    }

    pub fn set_state(&mut self, gl: &glow::Context, state: &GlState) {
        self.depth_test(gl, state.depth.enabled);
        self.depth_func(gl, state.depth.func);
        self.depth_mask(gl, state.depth.mask);
        self.culling(gl, state.cull.enabled);
        self.cull_face(gl, state.cull.face);
        self.front_face(gl, state.cull.front_face);
        self.blending(gl, state.blend.enabled);
        self.blend_func_separate(
            gl,
            state.blend.src_rgb,
            state.blend.dst_rgb,
            state.blend.src_alpha,
            state.blend.dst_alpha,
        );
        self.blend_equation(gl, state.blend.equation_rgb, state.blend.equation_alpha);
        self.stencil_test(gl, state.stencil.enabled);
        self.stencil_func(gl, state.stencil.func, state.stencil.reference, state.stencil.mask);
        self.stencil_op(gl, state.stencil.fail_op, state.stencil.z_fail_op, state.stencil.z_pass_op);
        self.scissor_test(gl, state.raster.scissor_test);
        self.scissor_box(gl, state.raster.scissor_box);
        self.viewport(gl, state.raster.viewport);
    }

    // depth

    pub fn depth_test(&mut self, gl: &glow::Context, value: bool) {
        if self.state.depth.enabled != value {
            self.state.depth.enabled = value;
            unsafe {
                if value { gl.enable(glow::DEPTH_TEST); }
                else     { gl.disable(glow::DEPTH_TEST); }
            }
        }
    }

    pub fn depth_func(&mut self, gl: &glow::Context, func: u32) {
        if self.state.depth.func != func {
            self.state.depth.func = func;
            unsafe { gl.depth_func(func); }
        }
    }

    pub fn depth_mask(&mut self, gl: &glow::Context, value: bool) {
        if self.state.depth.mask != value {
            self.state.depth.mask = value;
            unsafe { gl.depth_mask(value); }
        }
    }

    // culling

    pub fn culling(&mut self, gl: &glow::Context, value: bool) {
        if self.state.cull.enabled != value {
            self.state.cull.enabled = value;
            unsafe {
                if value { gl.enable(glow::CULL_FACE); }
                else     { gl.disable(glow::CULL_FACE); }
            }
        }
    }

    pub fn cull_face(&mut self, gl: &glow::Context, face: u32) {
        if self.state.cull.face != face {
            self.state.cull.face = face;
            unsafe { gl.cull_face(face); }
        }
    }

    pub fn front_face(&mut self, gl: &glow::Context, winding: u32) {
        if self.state.cull.front_face != winding {
            self.state.cull.front_face = winding;
            unsafe { gl.front_face(winding); }
        }
    }

    // blending

    pub fn blending(&mut self, gl: &glow::Context, value: bool) {
        if self.state.blend.enabled != value {
            self.state.blend.enabled = value;
            unsafe {
                if value { gl.enable(glow::BLEND); }
                else     { gl.disable(glow::BLEND); }
            }
        }
    }

    pub fn blend_func_both(&mut self, gl: &glow::Context, src: u32, dst: u32) {
        self.blend_func_separate(gl, src, dst, src, dst);
    }

    pub fn blend_func_separate(
        &mut self,
        gl: &glow::Context,
        src_rgb: u32,
        dst_rgb: u32,
        src_alpha: u32,
        dst_alpha: u32,
    ) {
        let b = &mut self.state.blend;
        if b.src_rgb != src_rgb
            || b.dst_rgb != dst_rgb
            || b.src_alpha != src_alpha
            || b.dst_alpha != dst_alpha
        {
            b.src_rgb = src_rgb;
            b.dst_rgb = dst_rgb;
            b.src_alpha = src_alpha;
            b.dst_alpha = dst_alpha;
            unsafe { gl.blend_func_separate(src_rgb, dst_rgb, src_alpha, dst_alpha); }
        }
    }

    pub fn blend_equation(&mut self, gl: &glow::Context, eq_rgb: u32, eq_alpha: u32) {
        let b = &mut self.state.blend;
        if b.equation_rgb != eq_rgb || b.equation_alpha != eq_alpha {
            b.equation_rgb = eq_rgb;
            b.equation_alpha = eq_alpha;
            unsafe { gl.blend_equation_separate(eq_rgb, eq_alpha); }
        }
    }

    // stencil

    pub fn stencil_test(&mut self, gl: &glow::Context, value: bool) {
        if self.state.stencil.enabled != value {
            self.state.stencil.enabled = value;
            unsafe {
                if value { gl.enable(glow::STENCIL_TEST); }
                else     { gl.disable(glow::STENCIL_TEST); }
            }
        }
    }

    pub fn stencil_func(&mut self, gl: &glow::Context, func: u32, reference: i32, mask: u32) {
        let s = &mut self.state.stencil;
        if s.func != func || s.reference != reference || s.mask != mask {
            s.func = func;
            s.reference = reference;
            s.mask = mask;
            unsafe { gl.stencil_func(func, reference, mask); }
        }
    }

    pub fn stencil_op(&mut self, gl: &glow::Context, fail: u32, z_fail: u32, z_pass: u32) {
        let s = &mut self.state.stencil;
        if s.fail_op != fail || s.z_fail_op != z_fail || s.z_pass_op != z_pass {
            s.fail_op = fail;
            s.z_fail_op = z_fail;
            s.z_pass_op = z_pass;
            unsafe { gl.stencil_op(fail, z_fail, z_pass); }
        }
    }

    // raster / viewport

    pub fn scissor_test(&mut self, gl: &glow::Context, value: bool) {
        if self.state.raster.scissor_test != value {
            self.state.raster.scissor_test = value;
            unsafe {
                if value { gl.enable(glow::SCISSOR_TEST); }
                else     { gl.disable(glow::SCISSOR_TEST); }
            }
        }
    }

    pub fn scissor_box(&mut self, gl: &glow::Context, box_: [i32; 4]) {
        if self.state.raster.scissor_box != box_ {
            self.state.raster.scissor_box = box_;
            unsafe { gl.scissor(box_[0], box_[1], box_[2], box_[3]); }
        }
    }

    pub fn viewport(&mut self, gl: &glow::Context, vp: [i32; 4]) {
        if self.state.raster.viewport != vp {
            self.state.raster.viewport = vp;
            unsafe { gl.viewport(vp[0], vp[1], vp[2], vp[3]); }
        }
    }
}

impl Default for GlStateManager {
    fn default() -> Self {
        Self::new()
    }
}


pub struct StateSnapshot {
    saved: GlState,
}

impl StateSnapshot {
    pub fn new(state: &GlStateManager) -> Self {
        Self {
            saved: state.snapshot(),
        }
    }

    pub fn restore(self, state: &mut GlStateManager, gl: &glow::Context) {
        state.set_state(gl, &self.saved);
    }

    pub fn saved(&self) -> &GlState {
        &self.saved
    }
}