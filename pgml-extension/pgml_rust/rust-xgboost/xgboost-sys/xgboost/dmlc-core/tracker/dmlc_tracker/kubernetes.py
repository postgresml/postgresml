#!/usr/bin/env python3
"""
DMLC submission script by kubernetes

One need to make sure kubectl-able.
"""
from __future__ import absolute_import

import os
from os import path
import sys
import uuid
import logging
from kubernetes import client, config
from . import tracker
import yaml

template_volume = {
        "name":""
        }
template_volumemount = {
        "mountPath":"",
        "name":""
        }
template_resouce = {
        "requests":{},
        "limits":{}
        }
sched_port = 9091
def create_svc_manifest(name, port, target_port):
    spec = client.V1ServiceSpec(
            selector={"app": name},
            ports=[client.V1ServicePort(protocol="TCP", port=port, target_port=target_port)]
            )
    service = client.V1Service( metadata=client.V1ObjectMeta(name=name), spec=spec)
    return service
def create_sched_svc_manifest(name, port):
    return create_svc_manifest(name, port, port)

def create_job_manifest(envs, commands, name, image, template_file):
    if template_file is not None:
        with open( template_file ) as f:
            job=yaml.safe_load(f)
            job["metadata"]["name"]=name
            job["spec"]["template"]["metadata"]["labels"]["app"]=name
            job["spec"]["template"]["spec"]["containers"][0]["image"]=image
            job["spec"]["template"]["spec"]["containers"][0]["command"]=commands
            job["spec"]["template"]["spec"]["containers"][0]["name"]=name
            job["spec"]["template"]["spec"]["containers"][0]["env"]=envs
            job["spec"]["template"]["spec"]["containers"][0]["command"]=commands
    else:
        container=client.V1Container(image=image, command=commands, name=name, env=envs)
        pod_temp=client.V1PodTemplateSpec(
                spec=client.V1PodSpec(restart_policy="OnFailure", containers=[container]),
                metadata=client.V1ObjectMeta(name=name, labels={"app":name})
                )
        job=client.V1Job(
                api_version="batch/v1",
                kind="Job",
                spec=client.V1JobSpec(template=pod_temp),
                metadata=client.V1ObjectMeta(name=name)
                )
    return job

def create_ps_manifest( ps_id, ps_num, job_name, envs, image, commands, template_file ):
    envs.append( client.V1EnvVar( name="DMLC_SERVER_ID", value=ps_id ))
    envs.append( client.V1EnvVar( name="DMLC_ROLE", value="server" ))
    if job_name is not None:
        name = "mx-" + job_name + "-server-" + ps_id
    else:
        name = "mx-server-" + ps_id
    return create_job_manifest(envs, commands, name, image, template_file )

def create_wk_manifest( wk_id, wk_num, ps_num, job_name, envs, image, commands, template_file ):
    envs.append( client.V1EnvVar( name="DMLC_WORKER_ID", value=wk_id ))
    envs.append( client.V1EnvVar( name="DMLC_SERVER_ID", value="0" ))
    envs.append( client.V1EnvVar( name="DMLC_ROLE", value="worker" ))
    if job_name is not None:
        name = "mx-" + job_name + "-worker-" + wk_id
    else:
        name = "mx-worker-" + wk_id
    return create_job_manifest(envs, commands, name, image, template_file )

def create_sched_job_manifest( wk_num, ps_num, envs, image,  commands):
    envs.append( client.V1EnvVar( name="DMLC_ROLE", value="scheduler" ))
    name = ""
    for i in envs:
        if i.name == "DMLC_PS_ROOT_URI":
            name = i.value
            break
    return create_job_manifest(envs, commands, name, image, None )
    
def create_env(root_uri, root_port, sv_num, wk_num ):
    envs = []
    envs.append( client.V1EnvVar( name="DMLC_PS_ROOT_URI", value=root_uri))
    envs.append( client.V1EnvVar( name="DMLC_PS_ROOT_PORT", value=str(root_port)))
    envs.append( client.V1EnvVar( name="DMLC_NUM_SERVER", value=str(sv_num)))
    envs.append( client.V1EnvVar( name="DMLC_NUM_WORKER", value=str(wk_num)))
    return envs
 

def submit(args):
    def kubernetes_submit(nworker, nserver, pass_envs):
        sv_image = args.kube_server_image
        wk_image = args.kube_worker_image
        if args.jobname is not None:
            r_uri = "mx-" + args.jobname + "-sched"
        else:
            r_uri = "mx-sched"
        r_port = 9091
        sd_envs = create_env( r_uri, r_port, nserver, nworker )
        mn_jobs = []
        mn_sh_job = create_sched_job_manifest( str(nworker), str(nserver), sd_envs, sv_image, args.command)
        mn_sh_svc = create_sched_svc_manifest(r_uri, r_port)
        
        for i in range(nserver):
            envs = create_env( r_uri, r_port, nserver, nworker )
            mn_sv = create_ps_manifest( str(i), str(nserver), args.jobname, envs, sv_image, args.command, args.kube_server_template )
            mn_jobs.append(mn_sv)

        for i in range(nworker):
            envs = create_env( r_uri, r_port, nserver, nworker )
            mn_wk = create_wk_manifest( str(i), str(nworker), str(nserver), args.jobname, envs, wk_image, args.command, args.kube_worker_template )
            mn_jobs.append(mn_wk)

        config.load_kube_config()
        k8s_coreapi = client.CoreV1Api()
        k8s_batch = client.BatchV1Api()
        resp = k8s_batch.create_namespaced_job(namespace=args.kube_namespace, body=mn_sh_job)
        print( resp.kind + " " + resp.metadata.name +" is created." )
        resp = k8s_coreapi.create_namespaced_service(namespace="default", body=mn_sh_svc)
        print( resp.kind + " " + resp.metadata.name +" is created." )
        for m in mn_jobs:
            resp = k8s_batch.create_namespaced_job(
                    body=m, namespace="default")
            print( resp.kind + " " + resp.metadata.name +" is created." )


        return kubernetes_submit

    tracker.submit(args.num_workers, args.num_servers,
                   fun_submit=kubernetes_submit,
                   pscmd="echo \"To check each log, try 'kubectl logs job/{{role}}-{{jobname}}-{{workerID}}'\"")
