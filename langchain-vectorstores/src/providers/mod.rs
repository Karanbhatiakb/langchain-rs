#![allow(dead_code)]

#[cfg(feature = "activeloop")]
pub mod activeloop;

#[cfg(feature = "alibabacloud_opensearch")]
pub mod alibabacloud_opensearch;

#[cfg(feature = "alibabacloud_search")]
pub mod alibabacloud_search;

#[cfg(feature = "alicloud_search")]
pub mod alicloud_search;

#[cfg(feature = "alicloud_tablestore")]
pub mod alicloud_tablestore;

#[cfg(feature = "amazon_neptune")]
pub mod amazon_neptune;

#[cfg(feature = "analyticdb")]
pub mod analyticdb;

#[cfg(feature = "analyticdb_postgres")]
pub mod analyticdb_postgres;

#[cfg(feature = "aspnet_memories")]
pub mod aspnet_memories;

#[cfg(feature = "atlas")]
pub mod atlas;

#[cfg(feature = "aws_kendra")]
pub mod aws_kendra;

#[cfg(feature = "aws_neptune")]
pub mod aws_neptune;

#[cfg(feature = "aws_opensearch_serverless")]
pub mod aws_opensearch_serverless;

#[cfg(feature = "aws_titandb")]
pub mod aws_titandb;

#[cfg(feature = "azure_ai_search")]
pub mod azure_ai_search;

#[cfg(feature = "azure_cognitive_search")]
pub mod azure_cognitive_search;

#[cfg(feature = "azure_cosmos_db")]
pub mod azure_cosmos_db;

#[cfg(feature = "azure_cosmosdb")]
pub mod azure_cosmosdb;

#[cfg(feature = "azure_search")]
pub mod azure_search;

#[cfg(feature = "bageldb")]
pub mod bageldb;

#[cfg(feature = "baidu_vector_search")]
pub mod baidu_vector_search;

#[cfg(feature = "baiducloud_vectorsearch")]
pub mod baiducloud_vectorsearch;

#[cfg(feature = "base")]
pub mod base;

#[cfg(feature = "cassandra_vector")]
pub mod cassandra_vector;

#[cfg(feature = "chroma_cloud")]
pub mod chroma_cloud;

#[cfg(feature = "chroma_embedded")]
pub mod chroma_embedded;

#[cfg(feature = "chroma_multi_modal")]
pub mod chroma_multi_modal;

#[cfg(feature = "clarifai")]
pub mod clarifai;

#[cfg(feature = "clickhouse")]
pub mod clickhouse;

#[cfg(feature = "cloudflare_vectorize")]
pub mod cloudflare_vectorize;

#[cfg(feature = "confluence_vector")]
pub mod confluence_vector;

#[cfg(feature = "cosmos_db_mongo")]
pub mod cosmos_db_mongo;

#[cfg(feature = "cosmosdb_nosql")]
pub mod cosmosdb_nosql;

#[cfg(feature = "couchbase_vector")]
pub mod couchbase_vector;

#[cfg(feature = "dashvector")]
pub mod dashvector;

#[cfg(feature = "datastax_vector")]
pub mod datastax_vector;

#[cfg(feature = "db2_vector")]
pub mod db2_vector;

#[cfg(feature = "dingo_db")]
pub mod dingo_db;

#[cfg(feature = "docarray")]
pub mod docarray;

#[cfg(feature = "duckdb_vector")]
pub mod duckdb_vector;

#[cfg(feature = "dynamodb_vector")]
pub mod dynamodb_vector;

#[cfg(feature = "ecloud_vector")]
pub mod ecloud_vector;

#[cfg(feature = "elastic_vector_search")]
pub mod elastic_vector_search;

#[cfg(feature = "elasticsearch_cloud")]
pub mod elasticsearch_cloud;

#[cfg(feature = "file_store")]
pub mod file_store;

#[cfg(feature = "google_bigquery_vector")]
pub mod google_bigquery_vector;

#[cfg(feature = "google_cloud_console")]
pub mod google_cloud_console;

#[cfg(feature = "google_cloud_matching_engine")]
pub mod google_cloud_matching_engine;

#[cfg(feature = "google_vertexai_vearc")]
pub mod google_vertexai_vearc;

#[cfg(feature = "hana_vector")]
pub mod hana_vector;

#[cfg(feature = "hologres")]
pub mod hologres;

#[cfg(feature = "huawei_cloud_search")]
pub mod huawei_cloud_search;

#[cfg(feature = "jaguar_vector")]
pub mod jaguar_vector;

#[cfg(feature = "json_store")]
pub mod json_store;

#[cfg(feature = "kdb_ai_cloud")]
pub mod kdb_ai_cloud;

#[cfg(feature = "kdbai")]
pub mod kdbai;

#[cfg(feature = "lantern")]
pub mod lantern;

#[cfg(feature = "llama_vision")]
pub mod llama_vision;

#[cfg(feature = "llamaindex")]
pub mod llamaindex;

#[cfg(feature = "logseq")]
pub mod logseq;

#[cfg(feature = "marqo")]
pub mod marqo;

#[cfg(feature = "maxdb")]
pub mod maxdb;

#[cfg(feature = "meilisearch")]
pub mod meilisearch;

#[cfg(feature = "mgml")]
pub mod mgml;

#[cfg(feature = "milvus_multi_modal")]
pub mod milvus_multi_modal;

#[cfg(feature = "milvus_standalone")]
pub mod milvus_standalone;

#[cfg(feature = "myscale")]
pub mod myscale;

#[cfg(feature = "notional_vector")]
pub mod notional_vector;

#[cfg(feature = "open_search")]
pub mod open_search;

#[cfg(feature = "oracle_vector")]
pub mod oracle_vector;

#[cfg(feature = "pgembedding")]
pub mod pgembedding;

#[cfg(feature = "postgres_embedding")]
pub mod postgres_embedding;

#[cfg(feature = "rockset_vector")]
pub mod rockset_vector;

#[cfg(feature = "sklearn")]
pub mod sklearn;

#[cfg(feature = "tencent_vectordb")]
pub mod tencent_vectordb;

#[cfg(feature = "tigris")]
pub mod tigris;

#[cfg(feature = "timescale")]
pub mod timescale;

#[cfg(feature = "typesense")]
pub mod typesense;

#[cfg(feature = "usearch")]
pub mod usearch;

#[cfg(feature = "vald")]
pub mod vald;

#[cfg(feature = "vespa")]
pub mod vespa;

#[cfg(feature = "yandex_vector")]
pub mod yandex_vector;

#[cfg(feature = "zilliz")]
pub mod zilliz;

#[cfg(feature = "aerospike_vector")]
pub mod aerospike_vector;

#[cfg(feature = "alibaba_cloud_es")]
pub mod alibaba_cloud_es;

#[cfg(feature = "alibaba_cloud_milvus")]
pub mod alibaba_cloud_milvus;

#[cfg(feature = "alibaba_cloud_polar")]
pub mod alibaba_cloud_polar;

#[cfg(feature = "alibaba_cloud_rds")]
pub mod alibaba_cloud_rds;

#[cfg(feature = "amazon_documentdb")]
pub mod amazon_documentdb;

#[cfg(feature = "amazon_keyspaces")]
pub mod amazon_keyspaces;

#[cfg(feature = "amazon_memorydb")]
pub mod amazon_memorydb;

#[cfg(feature = "amazon_rds")]
pub mod amazon_rds;

#[cfg(feature = "appwrite")]
pub mod appwrite;

#[cfg(feature = "arango_vector")]
pub mod arango_vector;

#[cfg(feature = "arctic_vector")]
pub mod arctic_vector;

#[cfg(feature = "astra_vector")]
pub mod astra_vector;

#[cfg(feature = "aws_cloudsearch")]
pub mod aws_cloudsearch;

#[cfg(feature = "aws_elasticache")]
pub mod aws_elasticache;

#[cfg(feature = "aws_healthlake")]
pub mod aws_healthlake;

#[cfg(feature = "azure_digital_twins")]
pub mod azure_digital_twins;

#[cfg(feature = "azure_ml")]
pub mod azure_ml;

#[cfg(feature = "azure_mysql")]
pub mod azure_mysql;

#[cfg(feature = "azure_postgres_flexible")]
pub mod azure_postgres_flexible;

#[cfg(feature = "azure_sql_vector")]
pub mod azure_sql_vector;

#[cfg(feature = "cassandra_astra")]
pub mod cassandra_astra;

#[cfg(feature = "chroma_persistent")]
pub mod chroma_persistent;

#[cfg(feature = "chroma_http")]
pub mod chroma_http;

#[cfg(feature = "cloudflare_d1")]
pub mod cloudflare_d1;

#[cfg(feature = "cockroach_vector")]
pub mod cockroach_vector;

#[cfg(feature = "cosmos_cassandra")]
pub mod cosmos_cassandra;

#[cfg(feature = "couchbase_gsi")]
pub mod couchbase_gsi;

#[cfg(feature = "databricks_vector")]
pub mod databricks_vector;

#[cfg(feature = "dgraph_vector")]
pub mod dgraph_vector;

#[cfg(feature = "hana_cloud")]
pub mod hana_cloud;

#[cfg(feature = "hasura_vector")]
pub mod hasura_vector;

#[cfg(feature = "hazelcast")]
pub mod hazelcast;

#[cfg(feature = "hbase")]
pub mod hbase;

#[cfg(feature = "hdfs_vector")]
pub mod hdfs_vector;

#[cfg(feature = "helm")]
pub mod helm;

#[cfg(feature = "heroku_vector")]
pub mod heroku_vector;

#[cfg(feature = "honeycomb")]
pub mod honeycomb;

#[cfg(feature = "hpcc")]
pub mod hpcc;

#[cfg(feature = "hyper")]
pub mod hyper;

#[cfg(feature = "hyperion")]
pub mod hyperion;

#[cfg(feature = "hypertable")]
pub mod hypertable;

#[cfg(feature = "hypertrace")]
pub mod hypertrace;

#[cfg(feature = "ibm_cos_vector")]
pub mod ibm_cos_vector;

#[cfg(feature = "ibm_db2_vector")]
pub mod ibm_db2_vector;

#[cfg(feature = "ibm_informix")]
pub mod ibm_informix;

#[cfg(feature = "ibm_netezza")]
pub mod ibm_netezza;

#[cfg(feature = "igdb")]
pub mod igdb;

#[cfg(feature = "ignition")]
pub mod ignition;

#[cfg(feature = "immuta")]
pub mod immuta;

#[cfg(feature = "impala")]
pub mod impala;

#[cfg(feature = "infinidb")]
pub mod infinidb;

#[cfg(feature = "influx")]
pub mod influx;

#[cfg(feature = "informatica")]
pub mod informatica;

#[cfg(feature = "infoworks")]
pub mod infoworks;

#[cfg(feature = "ingraph")]
pub mod ingraph;

#[cfg(feature = "innodb")]
pub mod innodb;

#[cfg(feature = "innovator")]
pub mod innovator;

#[cfg(feature = "interbase")]
pub mod interbase;

#[cfg(feature = "intersystems")]
pub mod intersystems;

#[cfg(feature = "iondb")]
pub mod iondb;

#[cfg(feature = "janusgraph")]
pub mod janusgraph;

#[cfg(feature = "jdbc_vector")]
pub mod jdbc_vector;

#[cfg(feature = "json_vector")]
pub mod json_vector;

#[cfg(feature = "k_nn_int8")]
pub mod k_nn_int8;

#[cfg(feature = "kvrocks")]
pub mod kvrocks;

#[cfg(feature = "kyoto")]
pub mod kyoto;

#[cfg(feature = "lance_format")]
pub mod lance_format;

#[cfg(feature = "leveldb")]
pub mod leveldb;

#[cfg(feature = "lmdb")]
pub mod lmdb;

#[cfg(feature = "lsm")]
pub mod lsm;

#[cfg(feature = "m_accelerated")]
pub mod m_accelerated;

#[cfg(feature = "machbase")]
pub mod machbase;

#[cfg(feature = "magic")]
pub mod magic;

#[cfg(feature = "magneto")]
pub mod magneto;

#[cfg(feature = "manifest")]
pub mod manifest;

#[cfg(feature = "mapr")]
pub mod mapr;

#[cfg(feature = "maria_vector")]
pub mod maria_vector;

#[cfg(feature = "matrix")]
pub mod matrix;

#[cfg(feature = "mcs")]
pub mod mcs;

#[cfg(feature = "memcached")]
pub mod memcached;

#[cfg(feature = "memsql")]
pub mod memsql;

#[cfg(feature = "mercure")]
pub mod mercure;

#[cfg(feature = "metis")]
pub mod metis;

#[cfg(feature = "mimer")]
pub mod mimer;

#[cfg(feature = "monetdb")]
pub mod monetdb;

#[cfg(feature = "mongo_atlas")]
pub mod mongo_atlas;

#[cfg(feature = "mongo_realm")]
pub mod mongo_realm;

#[cfg(feature = "mql")]
pub mod mql;

#[cfg(feature = "mroonga")]
pub mod mroonga;

#[cfg(feature = "mssql_vector")]
pub mod mssql_vector;

#[cfg(feature = "mysql_vector")]
pub mod mysql_vector;

#[cfg(feature = "native")]
pub mod native;

#[cfg(feature = "nats")]
pub mod nats;

#[cfg(feature = "navigator")]
pub mod navigator;

#[cfg(feature = "ncache")]
pub mod ncache;

#[cfg(feature = "ndb")]
pub mod ndb;

#[cfg(feature = "neptune_analytics")]
pub mod neptune_analytics;

#[cfg(feature = "neptune_graph")]
pub mod neptune_graph;

#[cfg(feature = "nessie")]
pub mod nessie;

#[cfg(feature = "nestdb")]
pub mod nestdb;

#[cfg(feature = "ngr")]
pub mod ngr;

#[cfg(feature = "nimbus")]
pub mod nimbus;

#[cfg(feature = "nitros")]
pub mod nitros;

#[cfg(feature = "nkv")]
pub mod nkv;

#[cfg(feature = "nothink")]
pub mod nothink;

#[cfg(feature = "ns")]
pub mod ns;

#[cfg(feature = "ntfs")]
pub mod ntfs;

#[cfg(feature = "nus")]
pub mod nus;

#[cfg(feature = "o_base")]
pub mod o_base;

