use crate::kmath::*;
use glow::*;
use crate::rect::*;
use std::fmt;


/*

OK how do I want to express the transform?
for the world its like, I want from here to here in world to be on my screen
or rather, center here with this zoomage

for minimap its like i want 0..1 to be in this spot on screen

*/



#[derive(Clone, Copy)]
#[repr(C)]
struct Triangle3 {
    a: Vert3,
    b: Vert3,
    c: Vert3,
}

impl fmt::Debug for Triangle3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pos: ({},{},{}), ({},{},{}), ({},{},{}) colour: ({},{},{},{})", 
            self.a.pos.x,
            self.a.pos.y,
            self.a.pos.z,
            self.b.pos.x,
            self.b.pos.y,
            self.b.pos.z,
            self.c.pos.x,
            self.c.pos.y,
            self.c.pos.z,
            self.a.colour.x,
            self.a.colour.y,
            self.a.colour.z,
            self.a.colour.w,
         )
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct Vert3 {
    pos: Vec3,
    colour: Vec4,
}

pub struct Renderer {
    triangles: Vec<Triangle3>,
    vbo: NativeBuffer,
    vao: NativeVertexArray,

    pub top_left: Vec2,
    pub bot_right: Vec2,
}

impl Renderer {
    pub fn new(gl: &glow::Context, aspect_ratio: f32) -> Renderer {
        unsafe {
            
            // We construct a buffer and upload the data
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            // We now construct a vertex array to describe the format of the input buffer
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 4*7, 0);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 4*7, 4*3);
            gl.enable_vertex_attrib_array(1);
    
            Renderer {triangles: Vec::new(),
                vao,
                vbo,
                top_left: Vec2::new(0.0, 0.0),
                bot_right: Vec2::new(1.0, 1.0),
            }
        }
    }

    fn push_triangle(&mut self, mut tri: Triangle3) {
        let dims = self.bot_right - self.top_left;

        tri.a.pos.x  = (tri.a.pos.x - self.top_left.x) / dims.x;
        tri.a.pos.y  = (tri.a.pos.y - self.top_left.y) / dims.y;

        tri.b.pos.x  = (tri.b.pos.x - self.top_left.x) / dims.x;
        tri.b.pos.y  = (tri.b.pos.y - self.top_left.y) / dims.y;

        tri.c.pos.x  = (tri.c.pos.x - self.top_left.x) / dims.x;
        tri.c.pos.y  = (tri.c.pos.y - self.top_left.y) / dims.y;

        self.triangles.push(tri);
    }

    pub fn draw_rect(&mut self, r: Rect, colour: Vec4, depth: f32) {
        let v1 = Vert3 {
            pos: Vec3::new(r.x, r.y, depth),
            colour: colour,
        };
        let v2 = Vert3 {
            pos: Vec3::new(r.x, r.y + r.h, depth),
            colour: colour,
        };
        let v3 = Vert3 {
            pos: Vec3::new(r.x + r.w, r.y + r.h, depth),
            colour: colour,
        };
        let v4 = Vert3 {
            pos: Vec3::new(r.x + r.w, r.y, depth),
            colour: colour,
        };
        self.push_triangle(Triangle3{ a: v1, b: v4, c: v3 });
        self.push_triangle(Triangle3{ a: v1, b: v3, c: v2 });
    }

    pub fn draw_tile(&mut self, r: Rect, colourN: Vec3, colourE: Vec3, colourS: Vec3, colourW: Vec3, depth: f32, alpha: f32) {
        let top_depth_colour = Vec4::new(0.7, 0.7, 0.7, 0.5);
        let bot_depth_colour = Vec4::new(0.3, 0.3, 0.3, 0.5);
        let depth_thickness = 0.03;

        let corner1 = Vec3::new(r.x, r.y, depth);
        let corner2 = Vec3::new(r.x + r.w, r.y, depth);
        let corner3 = Vec3::new(r.x + r.w, r.y + r.h, depth);
        let corner4 = Vec3::new(r.x, r.y + r.h, depth);
        let center = Vec3::new(r.x + r.w/2.0, r.y + r.h/2.0, depth);

        let cn = colourN.with_w(alpha);
        let ce = colourE.with_w(alpha);
        let cs = colourS.with_w(alpha);
        let cw = colourW.with_w(alpha);

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner1, colour: cn},
            b: Vert3{pos: corner2, colour: cn},
            c: Vert3{pos: center, colour: cn},
        });

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner2, colour: ce},
            b: Vert3{pos: corner3, colour: ce},
            c: Vert3{pos: center, colour: ce},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner3, colour: cs},
            b: Vert3{pos: corner4, colour: cs},
            c: Vert3{pos: center, colour: cs},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner4, colour: cw},
            b: Vert3{pos: corner1, colour: cw},
            c: Vert3{pos: center, colour: cw},
        });
        self.draw_rect(r.child(0.0, 0.0, 1.0, depth_thickness), top_depth_colour, depth + 0.05);
        self.draw_rect(r.child(0.0, 0.0, depth_thickness, 1.0), top_depth_colour, depth + 0.05);
        self.draw_rect(r.child(0.0, 1.0 - depth_thickness, 1.0, depth_thickness), bot_depth_colour, depth + 0.05);
        self.draw_rect(r.child(1.0 - depth_thickness, 0.0, depth_thickness, 1.0), bot_depth_colour, depth + 0.05);
    }

    pub fn draw_tile_no_bevel(&mut self, r: Rect, colourN: Vec3, colourE: Vec3, colourS: Vec3, colourW: Vec3, depth: f32, alpha: f32) {

        let corner1 = Vec3::new(r.x, r.y, depth);
        let corner2 = Vec3::new(r.x + r.w, r.y, depth);
        let corner3 = Vec3::new(r.x + r.w, r.y + r.h, depth);
        let corner4 = Vec3::new(r.x, r.y + r.h, depth);
        let center = Vec3::new(r.x + r.w/2.0, r.y + r.h/2.0, depth);

        let cn = colourN.with_w(alpha);
        let ce = colourE.with_w(alpha);
        let cs = colourS.with_w(alpha);
        let cw = colourW.with_w(alpha);

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner1, colour: cn},
            b: Vert3{pos: corner2, colour: cn},
            c: Vert3{pos: center, colour: cn},
        });

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner2, colour: ce},
            b: Vert3{pos: corner3, colour: ce},
            c: Vert3{pos: center, colour: ce},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner3, colour: cs},
            b: Vert3{pos: corner4, colour: cs},
            c: Vert3{pos: center, colour: cs},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner4, colour: cw},
            b: Vert3{pos: corner1, colour: cw},
            c: Vert3{pos: center, colour: cw},
        });
    }

    pub fn draw_tile_reverse_bevel(&mut self, r: Rect, colourN: Vec3, colourE: Vec3, colourS: Vec3, colourW: Vec3, depth: f32, alpha: f32) {
        let top_depth_colour = Vec4::new(0.7, 0.7, 0.7, 0.5);
        let bot_depth_colour = Vec4::new(0.3, 0.3, 0.3, 0.5);
        let depth_thickness = 0.03;

        let corner1 = Vec3::new(r.x, r.y, depth);
        let corner2 = Vec3::new(r.x + r.w, r.y, depth);
        let corner3 = Vec3::new(r.x + r.w, r.y + r.h, depth);
        let corner4 = Vec3::new(r.x, r.y + r.h, depth);
        let center = Vec3::new(r.x + r.w/2.0, r.y + r.h/2.0, depth);

        let cn = colourN.with_w(alpha);
        let ce = colourE.with_w(alpha);
        let cs = colourS.with_w(alpha);
        let cw = colourW.with_w(alpha);

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner1, colour: cn},
            b: Vert3{pos: corner2, colour: cn},
            c: Vert3{pos: center, colour: cn},
        });

        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner2, colour: ce},
            b: Vert3{pos: corner3, colour: ce},
            c: Vert3{pos: center, colour: ce},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner3, colour: cs},
            b: Vert3{pos: corner4, colour: cs},
            c: Vert3{pos: center, colour: cs},
        });
        
        self.push_triangle(Triangle3 {
            a: Vert3{pos: corner4, colour: cw},
            b: Vert3{pos: corner1, colour: cw},
            c: Vert3{pos: center, colour: cw},
        });

        self.draw_rect(r.child(0.0, 0.0, 1.0, depth_thickness), bot_depth_colour, depth + 0.05);
        self.draw_rect(r.child(0.0, 0.0, depth_thickness, 1.0), bot_depth_colour, depth + 0.05);
        self.draw_rect(r.child(0.0, 1.0 - depth_thickness, 1.0, depth_thickness), bot_depth_colour, depth + 0.05);
        self.draw_rect(r.child(1.0 - depth_thickness, 0.0, depth_thickness, 1.0), bot_depth_colour, depth + 0.05);
    }

    pub fn clear(&mut self) {
        self.triangles.clear();
    }

    pub fn present(&mut self, gl: &glow::Context) {
        unsafe {
            let gpu_bytes: &[u8] = core::slice::from_raw_parts(
                self.triangles.as_ptr() as *const u8,
                3 * 4 * 7 * self.triangles.len(),
            ); // 3 for points in triangle, 4 for bytes in float, 6 for floats in vertex
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, gpu_bytes, glow::DYNAMIC_DRAW);
            gl.draw_arrays(glow::TRIANGLES, 0, self.triangles.len() as i32 * 3);
            //gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }
    }
}