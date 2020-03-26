#include "shared/greengrasssdk.h"

gg_error gg_global_init(uint32_t opt) {
    return GGE_SUCCESS;
}

gg_error gg_log(gg_log_level level, const char *format, ...) 
{
    return GGE_SUCCESS;
}

gg_error gg_request_init(gg_request *ggreq) 
{
    return GGE_SUCCESS;
}

gg_error gg_request_close(gg_request ggreq) 
{
    return GGE_SUCCESS;
}

gg_error gg_request_read(gg_request ggreq, void *buffer, size_t buffer_size,
                         size_t *amount_read) {

    return GGE_SUCCESS;
}

gg_error gg_runtime_start(gg_lambda_handler handler, uint32_t opt) 
{
    return GGE_SUCCESS;
}

gg_error gg_lambda_handler_read(void *buffer, size_t buffer_size,
                                size_t *amount_read) 
{
    return GGE_SUCCESS;
}
gg_error gg_lambda_handler_write_response(const void *response,
                                          size_t response_size)
{
    return GGE_SUCCESS;
}

gg_error gg_lambda_handler_write_error(const char *error_message) 
{
    return GGE_SUCCESS;
}

gg_error gg_get_secret_value(gg_request ggreq, const char *secret_id,
		const char *version_id, const char *version_stage,
		gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_invoke(gg_request ggreq, const gg_invoke_options *opts,
                   gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_publish_options_init(gg_publish_options *opts) 
{
    return GGE_SUCCESS;
}

gg_error gg_publish_options_free(gg_publish_options opts)
{
    return GGE_SUCCESS;
}

gg_error gg_publish_options_set_queue_full_policy(gg_publish_options opts,
        gg_queue_full_policy_options policy) 
{
    return GGE_SUCCESS;
}

gg_error gg_publish_with_options(gg_request ggreq, const char *topic,
        const void *payload, size_t payload_size, const gg_publish_options opts,
        gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_publish(gg_request ggreq, const char *topic, const void *payload,
                    size_t payload_size, gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_get_thing_shadow(gg_request ggreq, const char *thing_name,
                             gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_delete_thing_shadow(gg_request ggreq, const char *thing_name,
                                gg_request_result *result) 
{
    return GGE_SUCCESS;
}

gg_error gg_update_thing_shadow(gg_request ggreq, const char *thing_name,
                                const char *update_payload,
                                gg_request_result *result) 
{
    return GGE_SUCCESS;
}