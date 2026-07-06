#ifndef PROMPTOS_LLAMA_H
#define PROMPTOS_LLAMA_H

#include <stdint.h>

int promptos_llm_init(const char *model_path);
int promptos_llm_is_loaded(void);
int promptos_llm_compile(const char *input, char *output, int output_max_len);
int promptos_llm_unload(void);
int promptos_llm_download_model(char *model_path, int max_len);

#endif /* PROMPTOS_LLAMA_H */
