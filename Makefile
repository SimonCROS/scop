NAME			:= scop
SHADERS_GLSL	:= shaders/default.vert shaders/default.frag
SHADERS_SPV		:= shaders/default.vert.spv shaders/default.frag.spv

all: build $(NAME)

build:
	cargo build

$(NAME): target/debug/scop $(SHADERS_SPV)
	cp target/debug/scop $(NAME)

clean:
	cargo clean -p scop

fclean:
	cargo clean
	rm -f $(SHADERS_SPV)
	rm -f $(NAME)

re: fclean all

%.spv: %
	glslc -o $@ $<

.PHONY:		all clean fclean re
