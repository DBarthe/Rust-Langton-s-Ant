RUSTCRATES				=   langton-ant sdl

langton-ant_CRATE_DEPS	+=	sdl	 

sdl_ROOTDIR				=	rust-sdl/
sdl_TYPE				=	lib


include             rust-mk/rust.mk