cmake_minimum_required(VERSION 3.5)

project(greengrasssdkstub)

############################################################
# Create a library
############################################################

#Generate the shared library from the library sources
add_library(greengrasssdk SHARED 
    src/greengrasssdk.c
)
add_library(greengrasssdk::library ALIAS greengrasssdk)

set_target_properties(greengrasssdk PROPERTIES OUTPUT_NAME "aws-greengrass-core-sdk-c")
target_include_directories(greengrasssdk
    PUBLIC 
        ${PROJECT_SOURCE_DIR}/include
)

install(TARGETS greengrasssdk DESTINATION lib)
install(FILES include/shared/greengrasssdk.h DESTINATION include)


