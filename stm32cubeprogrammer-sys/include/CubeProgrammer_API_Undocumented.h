/**
 * @brief Undocumented API functions for STM32CubeProgrammer
 * Credits to: https://github.com/wervin/stm32cubeprog-rs
 */

#pragma once

#ifdef __cplusplus
extern "C"
{
#endif

#include <stdint.h>
#include "DeviceDataStructure.h"

#if (defined WIN32 || defined _WIN32 || defined WINCE)
#define CP_EXPORTS __declspec(dllexport)
#else
#define CP_EXPORTS
#endif

/**
 * \brief Write a core register.
 * \param reg : The register to write.
 * \param data     : The data to write.
 * \return 0 if the writing operation correctly finished, otherwise an error occurred.
 */
int writeCoreRegister(unsigned int reg, unsigned int data);

/**
 * \brief Read a core register.
 * \param reg : The register to read.
 * \param data     : The data read.
 * \return 0 if the reading operation correctly finished, otherwise an error occurred.
 */
int readCoreRegister(unsigned int reg, unsigned int *data);

#ifdef __cplusplus
}
#endif