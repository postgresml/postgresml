// Copyright by Contributors
extern "C" {
#include <errno.h>
#include <curl/curl.h>
#include <openssl/hmac.h>
#include <openssl/buffer.h>
#include <openssl/sha.h>
}
#include <dmlc/io.h>
#include <dmlc/logging.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <algorithm>
#include <ctime>
#include <sstream>
#include <iomanip>

#include "./s3_filesys.h"

namespace dmlc {
namespace io {
/*! \brief namespace for helper utils */
namespace s3 {
// simple XML parser
struct XMLIter {
  // content of xml
  const char *content_;
  // end of content
  const char *cend_;
  XMLIter()
      : content_(NULL), cend_(NULL) {
  }
  // constructor
  explicit XMLIter(const char *content)
      : content_(content) {
    cend_ = content_ + strlen(content_);
  }
  /*! \brief convert to string */
  inline std::string str(void) const {
    if (content_ >= cend_) return std::string("");
    return std::string(content_, cend_ - content_);
  }
  /*!
   * \brief get next value of corresponding key in xml string
   * \param key the key in xml field
   * \param value the return value if success
   * \return if the get is success
   */
  inline bool GetNext(const char *key,
                      XMLIter *value) {
    std::string begin = std::string("<") + key +">";
    std::string end = std::string("</") + key +">";
    const char *pbegin = strstr(content_, begin.c_str());
    if (pbegin == NULL || pbegin > cend_) return false;
    content_ = pbegin + begin.size();
    const char *pend = strstr(content_, end.c_str());
    CHECK(pend != NULL) << "bad xml format";
    value->content_ = content_;
    value->cend_ = pend;
    content_ = pend + end.size();
    return true;
  }
};

/*!
 * \brief Converts hash to hex representation
 * \param hash unsigned char array with hash
 * \param size size of hash
 * \return string in hex representation
 */
static std::string SHA256HashToHex(unsigned char *hash, int size) {
  CHECK_EQ(size, SHA256_DIGEST_LENGTH);
  std::stringstream ss;
  for (int i=0; i < SHA256_DIGEST_LENGTH; i++) {
    ss << std::hex << std::setw(2) << std::setfill('0') << static_cast<int>(hash[i]);
  }
  return ss.str();
}

/*!
 * \brief Generates hash of input as per SHA256 algorithm and converts it to hex representation
 * \param str input to hash
 * \return string with hex representation of SHA256 Hash
 */
static std::string SHA256Hex(const std::string &str) noexcept {
  if (str.empty()) return "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
  unsigned char hash[SHA256_DIGEST_LENGTH];
  SHA256_CTX sha256;
  SHA256_Init(&sha256);
  SHA256_Update(&sha256, str.c_str(), str.size());
  SHA256_Final(hash, &sha256);
  return SHA256HashToHex(hash, SHA256_DIGEST_LENGTH);
}

/*!
 * \brief Returns datetime in ISO8601 format
 * Example: 20131222T043039Z
 * \param time
 * \return datetime in above format as string
 */
static std::string GetDateISO8601(const std::time_t &t) noexcept {
  char buf[sizeof "YYYYMMDDTHHMMSSZ"];
  std::strftime(buf, sizeof buf, "%Y%m%dT%H%M%SZ", std::gmtime(&t));
  return std::string{buf};
}

/*!
 * \brief Returns datetime in YYYYMMDD format
 * Example: 20131222T043039Z
 * \param time
 * \return datetime in above format as string
 */
static std::string GetDateYYYYMMDD(const std::time_t &t) noexcept {
  char buf[sizeof "YYYYMMDD"];
  std::strftime(buf, sizeof buf, "%Y%m%d", std::gmtime(&t));
  return std::string{buf};
}

static void AddDefaultCanonicalHeaders(std::map<std::string, std::string> *canonical_headers,
                                       const time_t &curr_time,
                                       const std::string &s3_session_token,
                                       const std::string &data,
                                       bool addDataHash = false) {
  (*canonical_headers)["x-amz-date"] = GetDateISO8601(curr_time);
  if (s3_session_token != "") {
    (*canonical_headers)["x-amz-security-token"] = s3_session_token;
  }
  if (addDataHash) {
    (*canonical_headers)["x-amz-content-sha256"] = SHA256Hex(data);
  }
}


/*!
 * \brief Returns keys of canonical_headers separated with semicolon
 * as per AWS SIG4 authentication
 * \param canonical_headers
 * \return signedHeaders as a string
 */
static std::string GetSignedHeaders(const std::map<std::string, std::string> &canonical_headers) {
  std::ostringstream stream;
  for (auto it = canonical_headers.begin(); it != canonical_headers.end(); ++it) {
    if (it != canonical_headers.begin()) {
      stream << ";";
    }
    stream << it->first;
  }
  return stream.str();
}

/*!
 * Encoding as required by SIG4
 * \param str string to encode
 * \param encodeSlash whether or not to encode slash (/) character
 * \return
 */
std::string URIEncode(const std::string& str,
                      bool encodeSlash = true) {
  std::stringstream encoded_str;
  encoded_str << std::hex << std::uppercase << std::setfill('0');
  for (std::string::const_iterator it = str.begin(); it != str.end(); ++it) {
    char c = *it;
    if ((c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        (c >= '0' && c <= '9') ||
        c == '-' || c == '_' ||
        c == '.' || c == '~') {
      encoded_str << c;
    } else if (c == '/') {
      if (encodeSlash) {
        encoded_str << "%2F";
      } else {
        encoded_str << c;
      }
    } else {
      encoded_str << '%';
      encoded_str << std::setw(2) << static_cast<unsigned>(c);
    }
  }
  return encoded_str.str();
}

/*!
 * \brief creates query string from keys and values in params
 * \param params query keys and values
 * \param is_canonical whether or not to produce canonical query by URIEncoding
 * \return query as a string
 */
static std::string GetQueryMultipart(const std::map<std::string, std::string> &params,
                                     const bool is_canonical) {
  bool init_request = (params.find("uploads") != params.end());
  std::ostringstream stream;
  for (auto it = params.begin(); it != params.end(); ++it) {
    if (it != params.begin()) {
      stream << "&";
    }
    if (is_canonical) {
      stream << URIEncode(it->first) << "=" << URIEncode(it->second);
    } else {
      if (init_request) {
        stream << it->first;
      } else {
        stream << it->first << "=" << it->second;
      }
    }
  }
  return stream.str();
}

/*!
 * \brief Returns credential scope as per AWS SIG4 authentication for S3 requests
 * \param time
 * \param region s3 region
 * \return credential scope
 */
static std::string GetCredentialScope(const time_t &time, const std::string &region) {
  return GetDateYYYYMMDD(time) + "/" + region  + "/s3/aws4_request";
}

/*!
 * \brief Calculates SIG4 Signature for an AWS request
 * \param request_date
 * \param secret s3_secret_key
 * \param region s3_region
 * \param service AWS service name
 * \param string_to_sign
 * \return signature
 */
static std::string CalculateSig4Sign(const std::time_t &request_date,
                                     const std::string &secret,
                                     const std::string &region,
                                     const std::string &service,
                                     const std::string &string_to_sign) {
  const std::string key1{"AWS4" + secret};
  const std::string yyyymmdd = GetDateYYYYMMDD(request_date);

  unsigned char* kDate;
  unsigned int kDateLen;
  kDate = HMAC(EVP_sha256(), key1.c_str(), key1.size(),
               reinterpret_cast<const unsigned char*>(yyyymmdd.c_str()),
               yyyymmdd.size(), NULL, &kDateLen);

  unsigned char *kRegion;
  unsigned int kRegionLen;
  kRegion = HMAC(EVP_sha256(), kDate, kDateLen,
                 reinterpret_cast<const unsigned char*>(region.c_str()),
                 region.size(), NULL, &kRegionLen);

  unsigned char *kService;
  unsigned int kServiceLen;
  kService = HMAC(EVP_sha256(), kRegion, kRegionLen,
                  reinterpret_cast<const unsigned char*>(service.c_str()),
                  service.size(), NULL, &kServiceLen);

  const std::string AWS4_REQUEST{"aws4_request"};
  unsigned char *kSigning;
  unsigned int kSigningLen;
  kSigning = HMAC(EVP_sha256(), kService, kServiceLen,
                  reinterpret_cast<const unsigned char*>(AWS4_REQUEST.c_str()),
                  AWS4_REQUEST.size(), NULL, &kSigningLen);

  unsigned char *kSig;
  unsigned int kSigLen;
  kSig = HMAC(EVP_sha256(), kSigning, kSigningLen,
              reinterpret_cast<const unsigned char*>(string_to_sign.c_str()),
              string_to_sign.size(), NULL, &kSigLen);
  return SHA256HashToHex(kSig, SHA256_DIGEST_LENGTH);
}

/*!
 * \brief Builds HTTP request headers for SIG4 auth requests to AWS
 * \param sauth stream for auth header
 * \param sdate stream for date
 * \param stoken stream for token
 * \param scontent stream for content related headers
 * \param time
 * \param s3_access_id
 * \param s3_region
 * \param s3_session_token
 * \param canonical_headers
 * \param signature SIG4 signature
 * \param payload data to send as payload
 */
static void BuildRequestHeaders(std::ostringstream& sauth,
                                std::ostringstream& sdate,
                                std::ostringstream& stoken,
                                std::ostringstream& scontent,
                                const time_t& curr_time,
                                const std::string& s3_access_id,
                                const std::string& s3_region,
                                const std::string& s3_session_token,
                                const std::map<std::string, std::string>& canonical_headers,
                                const std::string& signature,
                                const std::string& payload) {
  sauth << "Authorization: AWS4-HMAC-SHA256 ";
  sauth << "Credential=" << s3_access_id << "/" << GetCredentialScope(curr_time, s3_region) << ",";
  sauth << "SignedHeaders=" << GetSignedHeaders(canonical_headers) << ",";
  sauth << "Signature=" << signature;
  sdate << "x-amz-date: " << GetDateISO8601(curr_time);
  stoken << "x-amz-security-token: " << s3_session_token;
  scontent << "x-amz-content-sha256: " << SHA256Hex(payload);
}

/*!
 * \brief Signs the request as per SIG4 Auth scheme
 * https://docs.aws.amazon.com/AmazonS3/latest/API/sigv4-auth-using-authorization-header.html
 * \param key s3_access_key
 * \param s3_region
 * \param method method of HTTP request
 * \param time
 * \param canonical_uri
 * \param canonical_query
 * \param canonical_headers
 * \param payload data to send as payload
 * return signature
 */
static std::string SignSig4(const std::string &key,
                            const std::string &s3_region,
                            const std::string &method,
                            const time_t &time,
                            const std::string &canonical_uri,
                            const std::string &canonical_query,
                            const std::map<std::string, std::string> &canonical_headers,
                            const std::string &payload) {
  std::ostringstream can_req;
  can_req << method << "\n";
  can_req << canonical_uri << "\n";
  can_req << canonical_query << "\n";
  for (const auto & header : canonical_headers) {
    can_req << header.first << ":" << header.second << "\n";
  }
  can_req << "\n";
  can_req << GetSignedHeaders(canonical_headers);
  can_req << "\n";
  can_req << SHA256Hex(payload);

  std::string canonical_request = can_req.str();
  std::string hash_request = SHA256Hex(canonical_request);
  std::ostringstream to_sign;
  to_sign << "AWS4-HMAC-SHA256" << "\n";
  to_sign << GetDateISO8601(time) << "\n";
  to_sign << GetCredentialScope(time, s3_region) << "\n";
  to_sign << hash_request;
  return CalculateSig4Sign(time, key, s3_region, "s3", to_sign.str());
}

// remove the beginning slash at name
inline const char *RemoveBeginSlash(const std::string &name) {
  const char *s = name.c_str();
  while (*s == '/') {
    ++s;
  }
  return s;
}
// find the error field of the header
inline bool FindHttpError(const std::string &header) {
  std::string hd, ret;
  int code;
  std::istringstream is(header);
  if (is >> hd >> code >> ret) {
    if (code == 206 || ret == "OK") {
      return false;
    } else if (ret == "Continue") {
      return false;
    }
  }
  return true;
}

// curl callback to write sstream
size_t WriteSStreamCallback(char *buf, size_t size, size_t count, void *fp) {
  static_cast<std::ostringstream*>(fp)->write(buf, size * count);
  return size * count;
}

// callback by curl to write to std::string
size_t WriteStringCallback(char *buf, size_t size, size_t count, void *fp) {
  size *= count;
  std::string *str = static_cast<std::string*>(fp);
  size_t len = str->length();
  str->resize(len + size);
  std::memcpy(BeginPtr(*str) + len, buf, size);
  return size;
}

std::string getEndpoint(std::string region_name) {
  // using if elseif chain switching region_name
  if (region_name == "us-east-1") {
    return "s3.amazonaws.com";
  } else if (region_name == "cn-north-1" || region_name == "cn-northwest-1") {
    return "s3."+ region_name + ".amazonaws.com.cn";
  } else {
    return "s3-" + region_name + ".amazonaws.com";
  }
}

// useful callback for reading memory
struct ReadStringStream {
  const char *dptr;
  size_t nleft;
  // constructor
  explicit ReadStringStream(const std::string &data) {
    dptr = BeginPtr(data);
    nleft = data.length();
  }
  // curl callback to write sstream
  static size_t Callback(char *buf, size_t size, size_t count, void *fp) {
    size *= count;
    ReadStringStream *s = static_cast<ReadStringStream*>(fp);
    size_t nread = std::min(size, s->nleft);
    std::memcpy(buf, s->dptr, nread);
    s->dptr += nread; s->nleft -= nread;
    return nread;
  }
};

/*!
 * \brief reader stream that can be used to read from CURL
 */
class CURLReadStreamBase : public SeekStream {
 public:
  virtual ~CURLReadStreamBase() {
    this->Cleanup();
  }
  virtual size_t Tell(void) {
    return curr_bytes_;
  }
  virtual bool AtEnd(void) const {
    return at_end_;
  }
  virtual void Write(const void *ptr, size_t size) {
    LOG(FATAL) << "CURL.ReadStream cannot be used for write";
  }
  // lazy seek function
  virtual void Seek(size_t pos) {
    if (curr_bytes_ != pos) {
      this->Cleanup();
      curr_bytes_ = pos;
    }
  }
  virtual size_t Read(void *ptr, size_t size);

 protected:
  CURLReadStreamBase()
      : mcurl_(NULL), ecurl_(NULL), slist_(NULL),
        read_ptr_(0), curr_bytes_(0), at_end_(false) {
    expect_file_size_ = 0;
  }
  /*!
   * \brief initialize the ecurl request,
   * \param begin_bytes the beginning bytes of the stream
   * \param ecurl a curl easy handle that can be used to set request
   * \param slist a curl slist handle that can be used to set headers
   */
  virtual void InitRequest(size_t begin_bytes,
                           CURL *ecurl,
                           curl_slist **slist) = 0;

 protected:
  // the total size of the file
  size_t expect_file_size_;

 private:
  /*!
   * \brief called by child class to initialize read
   * \param begin_bytes the beginning bytes of the stream
   */
  void Init(size_t begin_bytes);
  /*!
   * \brief cleanup the previous session for restart
   */
  void Cleanup(void);
  /*!
   * \brief try to fill the buffer with at least wanted bytes
   * \param want_bytes number of bytes we want to fill
   * \return number of remainning running curl handles
   */
  int FillBuffer(size_t want_bytes);
  // multi and easy curl handle
  CURL *mcurl_, *ecurl_;
  // slist needed by the program
  curl_slist *slist_;
  // data buffer
  std::string buffer_;
  // header buffer
  std::string header_;
  // data pointer to read position
  size_t read_ptr_;
  // current position in the stream
  size_t curr_bytes_;
  // mark end of stream
  bool at_end_;
};

// read data in
size_t CURLReadStreamBase::Read(void *ptr, size_t size) {
  // lazy initialize
  if (mcurl_ == NULL) Init(curr_bytes_);
  // check at end
  if (at_end_) return 0;

  size_t nleft = size;
  char *buf = reinterpret_cast<char*>(ptr);
  while (nleft != 0) {
    if (read_ptr_ == buffer_.length()) {
      read_ptr_ = 0; buffer_.clear();
      if (this->FillBuffer(nleft) == 0 && buffer_.length() == 0) {
        at_end_ = true;
        break;
      }
    }
    size_t nread = std::min(nleft, buffer_.length() - read_ptr_);
    std::memcpy(buf, BeginPtr(buffer_) + read_ptr_, nread);
    buf += nread; read_ptr_ += nread; nleft -= nread;
  }
  size_t read_bytes = size - nleft;
  curr_bytes_ += read_bytes;

  // safety check, re-establish connection if failure happens
  if (at_end_ && expect_file_size_ != 0 &&
      curr_bytes_ != expect_file_size_) {
    int nretry = 0;
    CHECK_EQ(buffer_.length(), 0U);
    while (true) {
      LOG(ERROR) << "Re-establishing connection to Amazon S3, retry " << nretry;
      size_t rec_curr_bytes = curr_bytes_;
      this->Cleanup();
      this->Init(rec_curr_bytes);
      if (this->FillBuffer(nleft) != 0) break;
      ++nretry;
      CHECK_LT(nretry, 50)
          << "Unable to re-establish connection to read full file"
          << " ,expect_file_size=" << expect_file_size_
          << " ,curr_bytes=" << curr_bytes_;
      // sleep 100ms
#ifdef _WIN32
      Sleep(100);
#else
      struct timeval wait = { 0, 100 * 1000 };
      select(0, NULL, NULL, NULL, &wait);
#endif
    }
  }
  return read_bytes;
}

// cleanup the previous sessions for restart
void CURLReadStreamBase::Cleanup() {
  if (mcurl_ != NULL) {
    curl_multi_remove_handle(mcurl_, ecurl_);
    curl_easy_cleanup(ecurl_);
    curl_multi_cleanup(mcurl_);
    mcurl_ = NULL;
    ecurl_ = NULL;
  }
  if (slist_ != NULL) {
    curl_slist_free_all(slist_);
    slist_ = NULL;
  }
  buffer_.clear(); header_.clear();
  curr_bytes_ = 0; at_end_ = false;
}

void CURLReadStreamBase::Init(size_t begin_bytes) {
  CHECK(mcurl_ == NULL && ecurl_ == NULL &&
        slist_ == NULL) << "must call init in clean state";
  // make request
  ecurl_ = curl_easy_init();
  this->InitRequest(begin_bytes, ecurl_, &slist_);
  CHECK(curl_easy_setopt(ecurl_, CURLOPT_WRITEFUNCTION, WriteStringCallback) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl_, CURLOPT_WRITEDATA, &buffer_) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl_, CURLOPT_HEADERFUNCTION, WriteStringCallback) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl_, CURLOPT_HEADERDATA, &header_) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl_, CURLOPT_NOSIGNAL, 1) == CURLE_OK);
  mcurl_ = curl_multi_init();
  CHECK(curl_multi_add_handle(mcurl_, ecurl_) == CURLM_OK);
  int nrun;
  curl_multi_perform(mcurl_, &nrun);
  CHECK(nrun != 0 || header_.length() != 0 || buffer_.length() != 0);
  // start running and check header
  this->FillBuffer(1);
  if (FindHttpError(header_)) {
    while (this->FillBuffer(buffer_.length() + 256) != 0) {}
    LOG(FATAL) << "Request Error:\n" << header_ << buffer_;
  }
  // setup the variables
  at_end_ = false;
  curr_bytes_ = begin_bytes;
  read_ptr_ = 0;
}

// fill the buffer with wanted bytes
int CURLReadStreamBase::FillBuffer(size_t nwant) {
  int nrun = 0;
  while (buffer_.length() < nwant) {
    // wait for the event of read ready
    fd_set fdread;
    fd_set fdwrite;
    fd_set fdexcep;
    FD_ZERO(&fdread);
    FD_ZERO(&fdwrite);
    FD_ZERO(&fdexcep);
    int maxfd = -1;

    timeval timeout;
    long curl_timeo;  // NOLINT(*)
    curl_multi_timeout(mcurl_, &curl_timeo);
    if (curl_timeo < 0) curl_timeo = 980;
    timeout.tv_sec = curl_timeo / 1000;
    timeout.tv_usec = (curl_timeo % 1000) * 1000;
    CHECK(curl_multi_fdset(mcurl_, &fdread, &fdwrite, &fdexcep, &maxfd) == CURLM_OK);
    int rc;
    if (maxfd == -1) {
#ifdef _WIN32
      Sleep(100);
      rc = 0;
#else
      struct timeval wait = { 0, 100 * 1000 };
      rc = select(0, NULL, NULL, NULL, &wait);
#endif
    } else {
      rc = select(maxfd + 1, &fdread, &fdwrite, &fdexcep, &timeout);
    }
    if (rc != -1) {
      CURLMcode ret = curl_multi_perform(mcurl_, &nrun);
      if (ret ==  CURLM_CALL_MULTI_PERFORM) continue;
      CHECK(ret == CURLM_OK);
      if (nrun == 0) break;
    }
  }

  // loop through all the subtasks in curl_multi_perform and look for errors
  struct CURLMsg *m;
  do {
    int msgq = 0;
    m = curl_multi_info_read(mcurl_, &msgq);
    if (m && (m->msg == CURLMSG_DONE)) {
      if (m->data.result != CURLE_OK) {
        LOG(INFO) << "request failed with error "
                  << curl_easy_strerror(m->data.result);
      }
    }
  } while (m);

  return nrun;
}
// End of CURLReadStreamBase functions

// singleton class for global initialization
struct CURLGlobal {
  CURLGlobal() {
    CHECK(curl_global_init(CURL_GLOBAL_DEFAULT) == CURLE_OK);
  }
  ~CURLGlobal() {
    curl_global_cleanup();
  }
};

// used for global initialization
static CURLGlobal curl_global;

/*! \brief reader stream that can be used to read */
class ReadStream : public CURLReadStreamBase {
 public:
  ReadStream(const URI &path,
             const std::string &s3_id,
             const std::string &s3_key,
             const std::string &s3_session_token,
             const std::string &s3_region,
             const std::string &s3_endpoint,
             const bool s3_verify_ssl,
             const bool s3_is_aws,
             size_t file_size)
      : path_(path), s3_id_(s3_id), s3_key_(s3_key), s3_session_token_(s3_session_token),
         s3_region_(s3_region), s3_endpoint_(s3_endpoint), s3_verify_ssl_(s3_verify_ssl),
         s3_is_aws_(s3_is_aws) {
    this->expect_file_size_ = file_size;
  }
  virtual ~ReadStream(void) {}

 protected:
  // implement InitRequest
  virtual void InitRequest(size_t begin_bytes,
                           CURL *ecurl,
                           curl_slist **slist);

 private:
  // path we are reading
  URI path_;
  // s3 access key and id
  std::string s3_id_, s3_key_, s3_session_token_, s3_region_, s3_endpoint_;
  bool s3_verify_ssl_, s3_is_aws_;
};

// initialize the reader at begin bytes
void ReadStream::InitRequest(size_t begin_bytes,
                             CURL *ecurl,
                             curl_slist **slist) {
  std::string payload;
  time_t curr_time = time(NULL);
  std::map<std::string, std::string> canonical_headers;
  AddDefaultCanonicalHeaders(&canonical_headers, curr_time, s3_session_token_, payload, true);
  std::ostringstream sauth, sdate, stoken, surl, scontent, srange;
  std::ostringstream result;
  std::string canonical_querystring;
  std::string canonical_uri;
  CHECK_EQ(path_.name.front(), '/');
  CHECK_NE(path_.host.front(), '/');
  if (s3_is_aws_ && path_.host.find('.', 0) == std::string::npos) {
    // use virtual host style if no period in host
    canonical_uri = URIEncode(path_.name, false);
    canonical_headers["host"] = path_.host + "." + s3::getEndpoint(s3_region_);
    surl << "https://" << canonical_headers["host"]
         << '/' << RemoveBeginSlash(path_.name);
  } else {
    canonical_uri = URIEncode("/" + path_.host + path_.name, false);
    canonical_headers["host"] = s3_endpoint_;
    surl << "https://" << s3_endpoint_ << '/' << path_.host << '/'
         << RemoveBeginSlash(path_.name);
  }
  std::string signature = SignSig4(s3_key_, s3_region_, "GET", curr_time,
                                   canonical_uri, canonical_querystring,
                                   canonical_headers, payload);
  BuildRequestHeaders(sauth, sdate, stoken, scontent,
                      curr_time, s3_id_, s3_region_, s3_session_token_,
                      canonical_headers, signature, payload);

  srange << "Range: bytes=" << begin_bytes << "-";
  *slist = curl_slist_append(*slist, sdate.str().c_str());
  *slist = curl_slist_append(*slist, scontent.str().c_str());
  *slist = curl_slist_append(*slist, srange.str().c_str());
  *slist = curl_slist_append(*slist, sauth.str().c_str());
  if (s3_session_token_ != "") {
    *slist = curl_slist_append(*slist, stoken.str().c_str());
  }
  CHECK(curl_easy_setopt(ecurl, CURLOPT_HTTPHEADER, *slist) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl, CURLOPT_URL, surl.str().c_str()) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl, CURLOPT_HTTPGET, 1L) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl, CURLOPT_HEADER, 0L) == CURLE_OK);
  CHECK(curl_easy_setopt(ecurl, CURLOPT_NOSIGNAL, 1) == CURLE_OK);
  if (!s3_verify_ssl_) {
    CHECK(curl_easy_setopt(ecurl, CURLOPT_SSL_VERIFYHOST, 0L) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl, CURLOPT_SSL_VERIFYPEER, 0L) == CURLE_OK);
  }
}

/*! \brief simple http read stream to check */
class HttpReadStream : public CURLReadStreamBase {
 public:
  explicit HttpReadStream(const URI &path)
      : path_(path) {}
  // implement InitRequest
  virtual void InitRequest(size_t begin_bytes,
                           CURL *ecurl,
                           curl_slist **slist) {
    CHECK(begin_bytes == 0)
        << " HttpReadStream: do not support Seek";
    CHECK(curl_easy_setopt(ecurl, CURLOPT_URL, path_.str().c_str()) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl, CURLOPT_NOSIGNAL, 1) == CURLE_OK);
  }

 private:
  URI path_;
};

class WriteStream : public Stream {
 public:
  WriteStream(const URI &path,
              const std::string &s3_id,
              const std::string &s3_key,
              const std::string &s3_session_token,
              const std::string &s3_region,
              const std::string &s3_endpoint,
              bool s3_verify_ssl,
              bool s3_is_aws)
      : path_(path), s3_id_(s3_id), s3_key_(s3_key), s3_session_token_(s3_session_token),
         s3_region_(s3_region), s3_endpoint_(s3_endpoint), s3_verify_ssl_(s3_verify_ssl),
         s3_is_aws_(s3_is_aws), closed_(false) {
    const char *buz = getenv("DMLC_S3_WRITE_BUFFER_MB");
    if (buz != NULL) {
      max_buffer_size_ = static_cast<size_t>(atol(buz)) << 20UL;
    } else {
      // 64 MB
      const size_t kDefaultBufferSize = 64 << 20UL;
      max_buffer_size_ = kDefaultBufferSize;
    }
    max_error_retry_ = 3;
    ecurl_ = curl_easy_init();
    this->Init();
  }
  virtual size_t Read(void *ptr, size_t size) {
    LOG(FATAL) << "S3.WriteStream cannot be used for read";
    return 0;
  }
  virtual void Write(const void *ptr, size_t size);
  // destructor
  virtual ~WriteStream() {
    this->Close();
  }

  /*! \brief Closes the write stream */
  virtual void Close() {
    if (!closed_) {
      closed_ = true;
      this->Upload(true);
      this->Finish();
      curl_easy_cleanup(ecurl_);
    }
  }

 private:
  // internal maximum buffer size
  size_t max_buffer_size_;
  // maximum time of retry when error occurs
  int max_error_retry_;
  // path we are reading
  URI path_;
  // s3 access key and id
  std::string s3_id_, s3_key_, s3_session_token_, s3_region_, s3_endpoint_;
  bool s3_verify_ssl_, s3_is_aws_;
  // easy curl handle used for the request
  CURL *ecurl_;
  // upload_id used by AWS
  std::string upload_id_;
  // write data buffer
  std::string buffer_;
  // etags of each part we uploaded
  std::vector<std::string> etags_;
  // part id of each part we uploaded
  std::vector<size_t> part_ids_;
  // whether the stream is closed
  bool closed_;
  /*!
   * \brief helper function to do http post request
   * \param method method to peform
   * \param url_args additional arguments in URL
   * \param url_args translated arguments to sign
   * \param content_type content type of the data
   * \param data data to post
   * \param out_header holds output Header
   * \param out_data holds output data
   */
  void Run(const std::string &method,
           const std::map<std::string, std::string> &params,
           const std::string &content_type,
           const std::string &data,
           std::string *out_header,
           std::string *out_data);
  /*!
   * \brief initialize the upload request
   */
  void Init(void);
  /*!
   * \brief upload the buffer to S3, store the etag
   * clear the buffer
   */
  void Upload(bool force_upload_even_if_zero_bytes = false);
  /*!
   * \brief commit the upload and finish the session
   */
  void Finish(void);
};

void WriteStream::Write(const void *ptr, size_t size) {
  size_t rlen = buffer_.length();
  buffer_.resize(rlen + size);
  std::memcpy(BeginPtr(buffer_) + rlen, ptr, size);
  if (buffer_.length() >= max_buffer_size_) {
    this->Upload();
  }
}

void WriteStream::Run(const std::string &method,
                      const std::map<std::string, std::string> &params,
                      const std::string &content_type,
                      const std::string &data,
                      std::string *out_header,
                      std::string *out_data) {
  CHECK(path_.host.length() != 0) << "bucket name not specified for s3 location";
  CHECK(path_.name.length() != 0) << "key name not specified for s3 location";
  time_t curr_time = time(NULL);
  std::map<std::string, std::string> canonical_headers;
  AddDefaultCanonicalHeaders(&canonical_headers, curr_time, s3_session_token_, data, true);
  std::string canonical_query = GetQueryMultipart(params, true);
  std::string canonical_uri;
  std::ostringstream sauth, sdate, stoken, surl, scontent;
  std::ostringstream rheader, rdata;
  if (s3_is_aws_ && path_.host.find('.', 0) == std::string::npos) {
    canonical_uri = URIEncode(path_.name, false);
    canonical_headers["host"] = path_.host + "." + s3::getEndpoint(s3_region_);
    surl << "https://" << canonical_headers["host"]
         << path_.name << "?" << GetQueryMultipart(params, false);
  } else {
    canonical_uri = URIEncode("/" + path_.host + path_.name, false);
    canonical_headers["host"] = s3_endpoint_;
    surl << "https://" << s3_endpoint_ << "/" << path_.host
         << path_.name << "?" << GetQueryMultipart(params, false);
  }
  std::string signature = SignSig4(s3_key_, s3_region_, method, curr_time,
                                   canonical_uri, canonical_query,
                                   canonical_headers, data);
  BuildRequestHeaders(sauth, sdate, stoken, scontent,
                      curr_time, s3_id_, s3_region_, s3_session_token_,
                      canonical_headers, signature, data);
  scontent << "\nContent-Type: "<< content_type;

  // list
  curl_slist *slist = NULL;
  slist = curl_slist_append(slist, sdate.str().c_str());
  slist = curl_slist_append(slist, scontent.str().c_str());
  if (!s3_session_token_.empty()) {
    slist = curl_slist_append(slist, stoken.str().c_str());
  }
  slist = curl_slist_append(slist, sauth.str().c_str());

  int num_retry = 0;
  while (true) {
    // helper for read string
    ReadStringStream ss(data);
    curl_easy_reset(ecurl_);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_HTTPHEADER, slist) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_URL, surl.str().c_str()) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_HEADER, 0L) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_WRITEFUNCTION, WriteSStreamCallback) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_WRITEDATA, &rdata) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_WRITEHEADER, WriteSStreamCallback) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_HEADERDATA, &rheader) == CURLE_OK);
    CHECK(curl_easy_setopt(ecurl_, CURLOPT_NOSIGNAL, 1) == CURLE_OK);
    if (!s3_verify_ssl_) {
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_SSL_VERIFYHOST, 0L) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_SSL_VERIFYPEER, 0L) == CURLE_OK);
    }
    if (method == "POST") {
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_POST, 0L) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_POSTFIELDSIZE, data.length()) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_POSTFIELDS, BeginPtr(data)) == CURLE_OK);
    } else if (method == "PUT") {
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_PUT, 1L) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_READDATA, &ss) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_INFILESIZE_LARGE, data.length()) == CURLE_OK);
      CHECK(curl_easy_setopt(ecurl_, CURLOPT_READFUNCTION, ReadStringStream::Callback) == CURLE_OK);
    }
    CURLcode ret = curl_easy_perform(ecurl_);
    if (ret != CURLE_OK) {
      LOG(INFO) << "request " << surl.str() << "failed with error "
                << curl_easy_strerror(ret) << " Progress "
                << etags_.size() << " uploaded " << " retry=" << num_retry;
      num_retry += 1;
      CHECK(num_retry < max_error_retry_) << " maximum retry time reached";
      curl_easy_cleanup(ecurl_);
      ecurl_ = curl_easy_init();
    } else {
      break;
    }
  }
  curl_slist_free_all(slist);
  *out_header = rheader.str();
  *out_data = rdata.str();
  if (FindHttpError(*out_header) ||
      out_data->find("<Error>") != std::string::npos) {
    LOG(FATAL) << "AWS S3 Error:\n" << *out_header << *out_data;
  }
}

void WriteStream::Init(void) {
  std::string rheader, rdata;
  std::map<std::string, std::string> params;
  params["uploads"] = "";
  Run("POST", params, "binary/octel-stream", "", &rheader, &rdata);
  XMLIter xml(rdata.c_str());
  XMLIter upid;
  CHECK(xml.GetNext("UploadId", &upid)) << "missing UploadId";
  upload_id_ = upid.str();
}

void WriteStream::Upload(bool force_upload_even_if_zero_bytes) {
  if (buffer_.length() == 0 && !force_upload_even_if_zero_bytes) return;
  std::string rheader, rdata;
  size_t partno = etags_.size() + 1;
  std::map<std::string, std::string> params;
  params["partNumber"] = std::to_string(partno);
  params["uploadId"] = upload_id_;
  Run("PUT", params, "binary/octel-stream", buffer_, &rheader, &rdata);
  const char *p = strstr(rheader.c_str(), "ETag: ");
  CHECK(p != NULL) << "cannot find ETag in header";
  p = strchr(p, '\"');
  CHECK(p != NULL) << "cannot find ETag in header";
  const char *end = strchr(p + 1, '\"');
  CHECK(end != NULL) << "cannot find ETag in header";

  etags_.push_back(std::string(p, end - p + 1));
  part_ids_.push_back(partno);
  buffer_.clear();
}

void WriteStream::Finish(void) {
  std::string rheader, rdata;
  std::map<std::string, std::string> params;
  params["uploadId"] = upload_id_;

  std::ostringstream sdata;
  sdata << "<CompleteMultipartUpload>\n";
  CHECK(etags_.size() == part_ids_.size());
  for (size_t i = 0; i < etags_.size(); ++i) {
    sdata << " <Part>\n"
          << "  <PartNumber>" << part_ids_[i] << "</PartNumber>\n"
          << "  <ETag>" << etags_[i] << "</ETag>\n"
          << " </Part>\n";
  }
  sdata << "</CompleteMultipartUpload>\n";

  Run("POST", params, "text/xml", sdata.str(), &rheader, &rdata);
}
}  // namespace s3

void S3FileSystem::ListObjects(const URI &path, std::vector<FileInfo> *out_list) {
  CHECK(path.host.length() != 0) << "bucket name not specified for s3 location";
  out_list->clear();
  using namespace s3;

  std::string next_token = "";
  std::string has_next_page = "false";

  do {
    time_t curr_time = time(NULL);
    std::map<std::string, std::string> canonical_headers;
    std::string payload;
    std::ostringstream sauth, sdate, stoken, surl, scontent;
    std::ostringstream result;
    std::string canonical_uri;
    std::string canonical_querystring;

    AddDefaultCanonicalHeaders(&canonical_headers, curr_time, s3_session_token_, payload, true);
    if (next_token == "") {
        canonical_querystring = "delimiter=%2F&prefix=" +
            URIEncode(std::string{RemoveBeginSlash(path.name)});
    } else {
        canonical_querystring = "delimiter=%2F&marker=" + URIEncode(std::string{next_token}) +
            "&prefix=" + URIEncode(std::string{RemoveBeginSlash(path.name)});
    }

    if (s3_is_aws_ && path.host.find('.', 0) == std::string::npos) {
      // use virtual host style if no period in host
      canonical_uri = "/";
      canonical_headers["host"] = path.host + "." + s3::getEndpoint(s3_region_);
      surl << "https://" << canonical_headers["host"]
           << "/?delimiter=/&prefix=" << RemoveBeginSlash(path.name);
    } else {
      canonical_uri = URIEncode("/" + path.host + "/", false);
      canonical_headers["host"] = s3_endpoint_;
      surl << "https://" << s3_endpoint_ << "/" << path.host << "/?delimiter=/&prefix="
           << RemoveBeginSlash(path.name);
    }

    if (next_token != "") {
        surl << "&marker=" << next_token;
    }

    std::string signature = SignSig4(s3_secret_key_, s3_region_, "GET", curr_time,
                                     canonical_uri, canonical_querystring,
                                     canonical_headers, payload);
    BuildRequestHeaders(sauth, sdate, stoken, scontent,
                        curr_time, s3_access_id_, s3_region_, s3_session_token_,
                        canonical_headers, signature, payload);

    // make request
    CURL *curl = curl_easy_init();
    curl_slist *slist = NULL;
    slist = curl_slist_append(slist, sdate.str().c_str());
    slist = curl_slist_append(slist, sauth.str().c_str());
    slist = curl_slist_append(slist, scontent.str().c_str());
    if (!s3_session_token_.empty()) {
      slist = curl_slist_append(slist, stoken.str().c_str());
    }
    char errbuf[CURL_ERROR_SIZE];
    CHECK(curl_easy_setopt(curl, CURLOPT_ERRORBUFFER, &errbuf) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_HTTPHEADER, slist) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_URL, surl.str().c_str()) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_HTTPGET, 1L) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteSStreamCallback) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_WRITEDATA, &result) == CURLE_OK);
    CHECK(curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1) == CURLE_OK);
    if (!s3_verify_ssl_) {
      CHECK(curl_easy_setopt(curl, CURLOPT_SSL_VERIFYHOST, 0L) == CURLE_OK);
      CHECK(curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0L) == CURLE_OK);
    }
    CHECK(curl_easy_perform(curl) == CURLE_OK) << "Error: " << errbuf;
    curl_slist_free_all(slist);
    curl_easy_cleanup(curl);

    // parse xml
    std::string ret = result.str();
    if (ret.find("<Error>") != std::string::npos) {
      LOG(FATAL) << ret;
    }

    {
      // get files
      XMLIter xml(ret.c_str());
      XMLIter data;
      CHECK(xml.GetNext("IsTruncated", &data)) << "missing IsTruncated";
      has_next_page = data.str();
    }

    {
      // get files
      XMLIter xml(ret.c_str());
      XMLIter data;

      if (xml.GetNext("NextMarker", &data)) {
        // If NextContinuationToken exists in the response, more requests needs
        // to be made to get the full list of objects.
        next_token = data.str();
      }

      while (xml.GetNext("Contents", &data)) {
        FileInfo info;
        info.path = path;
        XMLIter value;
        CHECK(data.GetNext("Key", &value));
        // add root path to be consistent with other filesys convention
        info.path.name = '/' + value.str();
        CHECK(data.GetNext("Size", &value));
        info.size = static_cast<size_t>(atol(value.str().c_str()));
        info.type = kFile;
        out_list->push_back(info);
      }
    }

    {
      // get directories
      XMLIter xml(ret.c_str());
      XMLIter data;
      while (xml.GetNext("CommonPrefixes", &data)) {
        FileInfo info;
        info.path = path;
        XMLIter value;
        CHECK(data.GetNext("Prefix", &value));
        // add root path to be consistent with other filesys convention
        info.path.name = '/' + value.str();
        info.size = 0; info.type = kDirectory;
        out_list->push_back(info);
      }
    }
  } while (has_next_page == "true");
}

S3FileSystem::S3FileSystem() {
  const char *isAWS = getenv("S3_IS_AWS");
  const char *keyid = getenv("S3_ACCESS_KEY_ID");
  const char *seckey = getenv("S3_SECRET_ACCESS_KEY");
  const char *token = getenv("S3_SESSION_TOKEN");
  const char *region = getenv("S3_REGION");
  const char *endpoint = getenv("S3_ENDPOINT");
  const char *verify_ssl = getenv("S3_VERIFY_SSL");

  if (keyid == NULL || (strcmp(keyid, "") == 0)) {
    keyid = getenv("AWS_ACCESS_KEY_ID");
  }
  if (seckey == NULL || (strcmp(seckey, "") == 0)) {
    seckey = getenv("AWS_SECRET_ACCESS_KEY");
  }
  if (token == NULL || (strcmp(token, "") == 0)) {
    token = getenv("AWS_SESSION_TOKEN");
  }
  if (region == NULL || (strcmp(region, "") == 0)) {
    region = getenv("AWS_REGION");
  }

  if (keyid == NULL) {
    LOG(FATAL) << "Need to set enviroment variable S3_ACCESS_KEY_ID to use S3";
  }
  if (seckey == NULL) {
    LOG(FATAL) << "Need to set enviroment variable S3_SECRET_ACCESS_KEY to use S3";
  }

  if (isAWS == NULL || (strcmp(isAWS, "1") == 0)) {
    s3_is_aws_ = true;
  } else {
    s3_is_aws_ = false;
  }
  if (region == NULL) {
    LOG(WARNING) << "No AWS Region set, using default region us-east-1.";
    LOG(WARNING) << "Need to set enviroment variable S3_REGION to set region.";
    s3_region_ = "us-east-1";
  } else if (strcmp(region, "") == 0) {
    LOG(WARNING) << "AWS Region was set to empty string, using default region us-east-1.";
    LOG(WARNING) << "Need to set enviroment variable S3_REGION to set region.";
    s3_region_ = "us-east-1";
  } else {
    s3_region_ = region;
  }

  s3_access_id_ = keyid;
  s3_secret_key_ = seckey;

  if (token != NULL) {
    s3_session_token_ = token;
  }
  if (endpoint == NULL || (strcmp(endpoint, "") == 0)) {
    s3_endpoint_ = s3::getEndpoint(s3_region_);
  } else {
    s3_endpoint_ = endpoint;
  }

  if (verify_ssl == NULL || (strcmp(verify_ssl, "1") == 0)) {
    s3_verify_ssl_ = true;
  } else {
    s3_verify_ssl_ = false;
  }
}

void S3FileSystem::SetCredentials(const std::string& s3_access_id,
                                  const std::string& s3_secret_key) {
  s3_access_id_ = s3_access_id;
  s3_secret_key_ = s3_secret_key;
}

bool S3FileSystem::TryGetPathInfo(const URI &path_, FileInfo *out_info) {
  URI path = path_;
  while (path.name.length() > 1 &&
         *path.name.rbegin() == '/') {
    path.name.resize(path.name.length() - 1);
  }
  std::vector<FileInfo> files;
  ListObjects(path,  &files);
  std::string pdir = path.name + '/';
  for (size_t i = 0; i < files.size(); ++i) {
    if (files[i].path.name == path.name) {
      *out_info = files[i]; return true;
    }
    if (files[i].path.name == pdir) {
      *out_info = files[i]; return true;
    }
  }
  return false;
}

FileInfo S3FileSystem::GetPathInfo(const URI &path) {
  CHECK(path.protocol == "s3://")
      << " S3FileSystem.ListDirectory";
  FileInfo info;
  CHECK(TryGetPathInfo(path, &info))
      << "S3FileSytem.GetPathInfo cannot find information about " + path.str();
  return info;
}
void S3FileSystem::ListDirectory(const URI &path, std::vector<FileInfo> *out_list) {
  CHECK(path.protocol == "s3://")
      << " S3FileSystem.ListDirectory";
  if (path.name[path.name.length() - 1] == '/') {
    ListObjects(path,  out_list);
    return;
  }
  std::vector<FileInfo> files;
  std::string pdir = path.name + '/';
  out_list->clear();
  ListObjects(path,  &files);
  if (path.name.empty()) {
    // then insert all files in the bucket
    out_list->insert(out_list->end(), files.begin(), files.end());
    return;
  }
  for (size_t i = 0; i < files.size(); ++i) {
    if (files[i].path.name == path.name) {
      CHECK(files[i].type == kFile);
      out_list->push_back(files[i]);
      return;
    }
    if (files[i].path.name == pdir) {
      CHECK(files[i].type == kDirectory);
      ListObjects(files[i].path, out_list);
      return;
    }
  }
}

Stream *S3FileSystem::Open(const URI &path, const char* const flag, bool allow_null) {
  using namespace std;
  if (!strcmp(flag, "r") || !strcmp(flag, "rb")) {
    return OpenForRead(path, allow_null);
  } else if (!strcmp(flag, "w") || !strcmp(flag, "wb")) {
    CHECK(path.protocol == "s3://") << " S3FileSystem.Open";
    return new s3::WriteStream(path, s3_access_id_, s3_secret_key_, s3_session_token_,
                               s3_region_, s3_endpoint_, s3_verify_ssl_, s3_is_aws_);
  } else {
    LOG(FATAL) << "S3FileSytem.Open do not support flag " << flag;
    return NULL;
  }
}

SeekStream *S3FileSystem::OpenForRead(const URI &path, bool allow_null) {
  // simple http read stream
  if (!allow_null && (path.protocol == "http://"|| path.protocol == "https://")) {
    return new s3::HttpReadStream(path);
  }
  CHECK(path.protocol == "s3://") << " S3FileSystem.Open";
  FileInfo info;
  if (TryGetPathInfo(path, &info) && info.type == kFile) {
    return new s3::ReadStream(path, s3_access_id_, s3_secret_key_, s3_session_token_,
                              s3_region_, s3_endpoint_, s3_verify_ssl_, s3_is_aws_, info.size);
  } else {
    CHECK(allow_null) << " S3FileSystem: fail to open \"" << path.str() << "\"";
    return NULL;
  }
}
}  // namespace io
}  // namespace dmlc
