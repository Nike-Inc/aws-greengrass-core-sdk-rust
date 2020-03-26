/** Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved. */

/**
 * @file greengrasssdk.h
 * @brief Definition of SDK interfaces.
 */
#ifndef _GREENGRASS_SDK_H_
#define _GREENGRASS_SDK_H_

#ifdef __cplusplus
extern "C" {
#endif

#include <stdlib.h>
#include <stdint.h>
#include <sys/types.h>

/***************************************
**          Greengrass Types          **
***************************************/

/**
 * @brief Greengrass SDK error enum
 *
 * Enumeration of return values from the gg_* functions within the SDK.
 */
typedef enum gg_error {
    /** Returned when success */
    GGE_SUCCESS = 0,
    /** Returned when process is out of memory */
    GGE_OUT_OF_MEMORY,
    /** Returned when input parameter is invalid */
    GGE_INVALID_PARAMETER,
    /** Returned when SDK is in an invalid state */
    GGE_INVALID_STATE,
    /** Returned when SDK encounters internal failure */
    GGE_INTERNAL_FAILURE,
    /** Returned when process gets signal to terminate */
    GGE_TERMINATE,

    GGE_RESERVED_MAX,
    GGE_RESERVED_PAD = 0x7FFFFFFF
} gg_error;

typedef struct _gg_request *gg_request;

/**
 * @brief Greengrass SDK request status enum
 *
 * Enumeration of status populated when **gg_invoke()**, **gg_publish()** or
 * **gg_xxx_thing_shadow()** function is called.
 */
typedef enum gg_request_status {
    /** function call returns expected payload type */
    GG_REQUEST_SUCCESS = 0,
    /** function call is successfull, however lambda responss with an error */
    GG_REQUEST_HANDLED,
    /** function call is unsuccessfull, lambda exits abnormally */
    GG_REQUEST_UNHANDLED,
    /** System encounters unknown error. Check logs for more details */
    GG_REQUEST_UNKNOWN,
    /** function call is throttled, try again */
    GG_REQUEST_AGAIN,

    GG_REQUEST_RESERVED_MAX,
    GG_REQUEST_RESERVED_PAD = 0x7FFFFFFF
} gg_request_status;

/**
 * @brief Describes result metadata after request is made
 * @param request_status the request status
 */
typedef struct gg_request_result {
    /** Status enum after request is made */
    gg_request_status request_status;
} gg_request_result;

/**
 * @brief Describes context when lambda handler is called
 * @param function_arn Null-terminated string full lambda ARN
 * @param client_context Null-terminated string of client context
 */
typedef struct gg_lambda_context {
    const char *function_arn;
    const char *client_context;
} gg_lambda_context;

/**
 * @brief Describes invocation type for lambda function
 */
typedef enum gg_invoke_type {
    /** Invoke the function asynchronously */
    GG_INVOKE_EVENT,
    /** Invoke the function synchronously (default) */
    GG_INVOKE_REQUEST_RESPONSE,

    GG_INVOKE_RESERVED_MAX,
    GG_INVOKE_RESERVED_PAD = 0x7FFFFFFF
} gg_invoke_type;

/**
 * @brief Flags set for the gg_runtime_start)(.., opt)
 */
typedef enum gg_runtime_opt {
    /** Start the runtime in a new thread. Runtime will exit if main thread exits */
	GG_RT_OPT_ASYNC = 0x1,
	GG_RT_OPT_RESERVED_PAD = 0x7FFFFFFF
} gg_runtime_opt;

/**
 * @brief Describes the options to invoke a target lambda
 *
 * @param function_arn Null-terminated string full lambda ARN to be invoked
 * @param customer_context base64-encoded null-terminated json string
 * @param qualifier Null-terminated string version of the function
 * @param type Specifiy whether a response is needed
 * @param payload Buffer to be sent to the invoked lambda
 * @param payload_size Size of payload buffer
 */
typedef struct gg_invoke_options {
    const char *function_arn;
    const char *customer_context;
    const char *qualifier;
    gg_invoke_type type;
    const void *payload;
    size_t payload_size;
} gg_invoke_options;

/**
 * @brief Describes the policy options to take when Greengrass's queue is full
 */
typedef enum gg_queue_full_policy_options {
    /** GGC will deliver messages to as many targets as possible **/
    GG_QUEUE_FULL_POLICY_BEST_EFFORT,
    /** GGC will either deliver messages to all targets and return request
     * status GG_REQUEST_SUCCESS or deliver to no targets and return a
     * request status GG_REQUEST_AGAIN **/
    GG_QUEUE_FULL_POLICY_ALL_OR_ERROR,

    GG_QUEUE_FULL_POLICY_RESERVED_MAX,
    GG_QUEUE_FULL_POLICY_RESERVED_PAD = 0x7FFFFFFF
} gg_queue_full_policy_options;

typedef struct _gg_publish_options *gg_publish_options;

/**
 * @brief Describes log levels could used in **gg_log()**
 */
typedef enum gg_log_level {
    GG_LOG_RESERVED_NOTSET,

    /** Debug */
    GG_LOG_DEBUG,
    /** Info */
    GG_LOG_INFO,
    /** Warn */
    GG_LOG_WARN,
    /** Error */
    GG_LOG_ERROR,
    /** Fatal. System will exist */
    GG_LOG_FATAL,

    GG_LOG_RESERVED_MAX,
    GG_LOG_RESERVED_PAD = 0x7FFFFFFF
} gg_log_level;

/***************************************
**            Global Methods          **
***************************************/


/**
 * @brief Initialize Greengrass internal global variables
 * @param opt Reserved for future use. Must be set to 0.
 * @return Greengrass error code
 * @note THIS IS NOT THREAD SAFE and must be called when there is only a single
 *       main thread executing.
 *       User must call this function before creating any threads.
 *       User must call this function before calling any other Greengrass
 *           function in this SDK.
 */
gg_error gg_global_init(uint32_t opt);

/***************************************
**           Logging Methods          **
***************************************/

/**
 * @brief Log message to Greengrass Core using similar syntax to printf
 * @param level Level of message that can be filtered based on settings
 * @param format Similar to printf
 * @param ... Similar to printf
 * @return Greengrass error code
 */
gg_error gg_log(gg_log_level level, const char *format, ...);

/***************************************
**         gg_request Methods         **
***************************************/

/**
 * @brief Initialize the context for managing the request
 * @param ggreq Pointer to context to be initialized
 * @return Greengrass error code
 * @note Need to call gg_request_close on ggreq when done using it
 */
gg_error gg_request_init(gg_request *ggreq);

/**
 * @brief Close a request context that was created by gg_request_init
 * @param ggreq Context to be closed
 * @return Greengrass error code
 */
gg_error gg_request_close(gg_request ggreq);

/**
 * @brief Read the data from a request. This method should be called
 *        till amount_read is zero.
 *
 * @param ggreq Provides context about the request
 * @param buffer Destination for read data
 * @param buffer_size Size of buffer
 * @param amount_read Destination for amount of data read into buffer
 * @return Greengrass error code
 */
gg_error gg_request_read(gg_request ggreq, void *buffer, size_t buffer_size,
                         size_t *amount_read);

/***************************************
**           Runtime Methods          **
***************************************/

/**
 * @brief Handler signature that will be called whenever a subscribed message
 *        is received
 * @param cxt Details about the lambda invocation
 */
typedef void (*gg_lambda_handler)(const gg_lambda_context *cxt);

/**
 * @brief Registers the lambda handler and start Greengrass lambda runtime
 *
 * @param handler Customer lambda code to be run when subscription is triggered
 * @param opt Mask flags of gg_runtime_opt options, 0 for default
 * @note Must be called. This uses and will overwrite the SIGTERM handler
 */
gg_error gg_runtime_start(gg_lambda_handler handler, uint32_t opt);

/**
 * @brief Read the data from the invoker of the lambda. This method should be called
 *        till amount_read is zero.
 *
 * @param buffer Destination for read data
 * @param buffer_size Size of buffer
 * @param amount_read Destination for amount of data read into buffer
 * @return Greengrass error code
 * @note This should only be used in the lambda handler
 */
gg_error gg_lambda_handler_read(void *buffer, size_t buffer_size,
                                size_t *amount_read);

/**
 * @brief Write response to the invoker of the lambda
 * @param response Response data to be written
 * @param response_size Amount of data stored in response
 * @return Greengrass error code
 * @note This should only be used in the lambda handler
 */
gg_error gg_lambda_handler_write_response(const void *response,
                                          size_t response_size);

/**
 * @brief Write error message to the invoker of the lambda
 * @param error_message Null-terminated string error message to be written
 * @return Greengrass error code
 * @note This should only be used in the lambda handler
 * @note The caller's invoke will return result GG_REQUEST_HANDLED
 *       in the gg_request_result struct instead of GG_REQUEST_SUCCESS.
 */
gg_error gg_lambda_handler_write_error(const char *error_message);

/***************************************
**     AWS Secrets Manager Methods    **
***************************************/

/**
 * @brief Get secret value for the given secret
 * @param ggreq Provides context about the request
 * @param secret_id Null-terminated string id which secret to get
 * @param version_id Null-terminated string version id which version to get
 * @param version_stage Optional null-terminated string version stage which stage to get
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_get_secret_value(gg_request ggreq, const char *secret_id,
		const char *version_id, const char *version_stage,
		gg_request_result *result);

/***************************************
**           Lambda Methods           **
***************************************/

/**
 * @brief Invoke a lambda with an optional payload
 * @param ggreq Provides context about the request
 * @param opts Describes the options for invoke
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_invoke(gg_request ggreq, const gg_invoke_options *opts,
                   gg_request_result *result);

/***************************************
**           AWS IoT Methods          **
***************************************/

/**
 * @brief Initialize the publish options
 * @param opts Pointer to publish options to be initialized
 * @return Greengrass error code
 * @note Need to call gg_publish_options_free on opts when done using it
 */
gg_error gg_publish_options_init(gg_publish_options *opts);

/**
 * @brief Free a publish options that was created by gg_publish_options_init
 * @param opts Publish options to be freed
 * @return Greengrass error code
 */
gg_error gg_publish_options_free(gg_publish_options opts);

/**
 * @brief Sets the queue full policy on a publish options
 * @param opts Publish options to be configured
 * @param policy Selected queue full policy to be set
 * @return Greengrass error code
 */
gg_error gg_publish_options_set_queue_full_policy(gg_publish_options opts,
        gg_queue_full_policy_options policy);

/**
 * @brief Publish a payload to a topic
 * @param ggreq Provides context about the request
 * @param topic Null-terminated string topic where to publish the payload
 * @param payload Data to be sent to the topic - caller will free
 * @param payload_size Size of payload buffer
 * @param opts Publish options that configure publish behavior
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_publish_with_options(gg_request ggreq, const char *topic,
        const void *payload, size_t payload_size, const gg_publish_options opts,
        gg_request_result *result);

/**
 * @brief Publish a payload to a topic
 * @param ggreq Provides context about the request
 * @param topic Null-terminated string topic where to publish the payload
 * @param payload Data to be sent to the topic - caller will free
 * @param payload_size Size of payload buffer
 * @param result Describes the result of the request
 * @return Greengrass error code
 * @note Calls gg_publish_with_options with opts==NULL
 */
gg_error gg_publish(gg_request ggreq, const char *topic, const void *payload,
                    size_t payload_size, gg_request_result *result);

/**
 * @brief Get thing shadow for thing name
 * @param ggreq Provides context about the request
 * @param thing_name Null-terminated string specifying thing shadow to get
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_get_thing_shadow(gg_request ggreq, const char *thing_name,
                             gg_request_result *result);

/**
 * @brief Update thing shadow for thing name
 * @param ggreq Provides context about the request
 * @param thing_name Null-terminated string specifying thing shadow to update
 * @param update_payload Null-terminated string to be updated in the shadow
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_update_thing_shadow(gg_request ggreq, const char *thing_name,
                                const char *update_payload,
                                gg_request_result *result);

/**
 * @brief Delete thing shadow for thing name
 * @param ggreq Provides context about the request
 * @param thing_name Null-terminated string specifying thing shadow to delete
 * @param thing_name Specifies which thing shadow should be deleted
 * @param result Describes the result of the request
 * @return Greengrass error code
 */
gg_error gg_delete_thing_shadow(gg_request ggreq, const char *thing_name,
                                gg_request_result *result);

#ifdef __cplusplus
}
#endif

#endif /* #ifndef _GREENGRASS_SDK_H_ */
