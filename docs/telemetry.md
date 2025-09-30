# Telemetry

The `cave` CLI includes optional telemetry to help improve the tool by collecting **anonymous usage data**. You can control telemetry via the configuration commands.

## Collected Data

When telemetry is enabled, the following data is collected:

- **user_id**: a randomly generated UUID (`Uuid::new_v4()`) to identify usage anonymously.  
- **time_execution**: execution duration in milliseconds.  
- **valid_result**: whether the code_aster execution is valid
- **timezone**: the local timezone of the machine. 
- **version**: the version of code\_aster used.  
- **id_docker**: the Docker image ID.


## Example Data

Here is an example of what a telemetry record looks like:

```json
{
  "user_id": "23806108-75e2-4fe7-8bcf-9b2da5659890",
  "time_execution": 7899,
  "valid_result": true,
  "timezone": "+02:00",
  "version": "17.2.13",
  "id_docker": "e0d78ea06146"
}
```

## Control Telemetry

To disable telemetry, run:

```bash
cave config disable-usage-tracking
```


