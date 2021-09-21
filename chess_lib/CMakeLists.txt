include(GNUInstallDirs)
set(SOURCES src/bitboard.c src/found_magics.c src/move_gen.c)

add_library(chess-util SHARED ${SOURCES})
set_target_properties(chess-util PROPERTIES VERSION 0.1 SOVERSION 1 PUBLIC_HEADER include/chess-util.h)
target_include_directories(chess-util PRIVATE src/ include/)
install(TARGETS chess-util
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        COMPONENT chess-util
        PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
        COMPONENT chess-util)

check_ipo_supported(RESULT supported OUTPUT error)
if (supported)
    message(STATUS "IPO / LTO supported")
    set_property(TARGET chess-util PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
else ()
    message(STATUS "IPO / LTO not supported: <${error}>")
endif ()

