use crate::{has_handle, transmutable_u32};
use gl::types::*;
use glam::Vec2;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Compute = gl::COMPUTE_SHADER,
    Vertex = gl::VERTEX_SHADER,
    TessControl = gl::TESS_CONTROL_SHADER,
    TessEval = gl::TESS_EVALUATION_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
}
transmutable_u32!(ShaderType);

pub struct Shader {
    handle: GLuint,
}
has_handle!(Shader);
// impl Drop for Shader {
//     fn drop(&mut self) {
//         unsafe {
//             gl::DeleteShader(self.as_handle());
//         }
//     }
// }

impl Shader {
    pub fn new_many_sources<const N: usize>(shader_type: ShaderType, sources: [&str; N]) -> Self {
        let handle = unsafe { gl::CreateShader(shader_type.to_u32()) };
        let lengths = sources.map(|s| s.len() as GLint);
        let sources = sources.map(|s| s.as_ptr().cast::<GLchar>());
        unsafe {
            gl::ShaderSource(handle, 1, sources.as_ptr(), lengths.as_ptr());
            gl::CompileShader(handle);
        }

        let mut success = 0;
        unsafe {
            gl::GetShaderiv(handle, gl::COMPILE_STATUS, &mut success);
        }

        if success == gl::FALSE.into() {
            eprintln!("Shader compilation failed:");
            let mut log_size = 0;
            unsafe {
                gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut log_size);
            }

            let mut info_log: Vec<u8> = Vec::with_capacity(log_size as usize);
            let mut bytes_written = 0;
            unsafe {
                gl::GetShaderInfoLog(
                    handle,
                    log_size,
                    &mut bytes_written,
                    info_log.as_mut_ptr().cast(),
                );
                gl::DeleteShader(handle);
                info_log.set_len(bytes_written as usize);
            }

            let info_log = String::from_utf8(info_log).unwrap();
            eprintln!(" | ");
            for line in info_log.lines() {
                eprintln!(" | {}", line);
            }
            eprintln!(" | ");
            panic!("Shader compilation failed.")
        }

        Self { handle }
    }

    pub fn new(shader_type: ShaderType, source: &str) -> Self {
        Self::new_many_sources(shader_type, [source])
    }
    #[inline]
    pub fn vertex(source: &str) -> Self {
        Self::new(ShaderType::Vertex, source)
    }
    #[inline]
    pub fn fragment(source: &str) -> Self {
        Self::new(ShaderType::Fragment, source)
    }
    #[inline]
    pub fn compute(source: &str) -> Self {
        Self::new(ShaderType::Compute, source)
    }
}

pub struct ShaderProgram {
    handle: GLuint,
}
has_handle!(ShaderProgram);
// impl Drop for ShaderProgram {
//     fn drop(&mut self) {
//         unsafe {
//             gl::DeleteProgram(self.as_handle());
//         }
//     }
// }

impl ShaderProgram {
    pub fn new<T>(shaders: T) -> Self
    where
        T: IntoIterator<Item = Shader>,
        for<'a> &'a T: IntoIterator<Item = &'a Shader>,
    {
        unsafe {
            let handle = gl::CreateProgram();
            for shader in &shaders {
                gl::AttachShader(handle, shader.as_handle());
            }
            gl::LinkProgram(handle);
            for shader in shaders {
                gl::DetachShader(handle, shader.as_handle());
                // drop(shader);
            }
            Self { handle }
        }
    }

    pub fn use_self(&self) {
        unsafe {
            gl::UseProgram(self.handle);
        }
    }

    pub fn set_uniform_vec2(&self, location: i32, data: Vec2) {
        unsafe { gl::ProgramUniform2f(self.handle, location, data.x, data.y) };
    }
}
