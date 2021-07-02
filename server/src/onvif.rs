extern crate onvif;
extern crate xsd_types;
extern crate yaserde;

use onvif::{schema, soap};
use url::Url;
use self::onvif::schema::transport::Transport;
use self::onvif::schema::ws_addr::{EndpointReferenceType, AttributedURIType};
use self::onvif::schema::b_2::subscribe::SubscriptionPolicyType;
use self::onvif::schema::b_2::AbsoluteOrRelativeTimeType;
use xsd_types::types as xs;
use std::str::FromStr;
use self::onvif::schema::b_2::AbsoluteOrRelativeTimeType::Duration;

use yaserde::{YaDeserialize, YaSerialize};

pub struct OnvifClient {
  devicemgmt: soap::client::Client,
  event: Option<soap::client::Client>,
}

impl OnvifClient {
  pub async fn new(base_uri: Url, username: &str, password: &str) -> Result<Self, String> {
    let creds = soap::client::Credentials {
      username: username.to_string(),
      password: password.to_string(),
    };
    let devicemgmt_uri = base_uri.join("onvif/device_service").unwrap();
    let mut out = Self {
      devicemgmt: soap::client::ClientBuilder::new(&devicemgmt_uri)
          .credentials(Some(creds.clone()))
          .build(),
      event: None,
    };
    let services =
        schema::devicemgmt::get_services(&out.devicemgmt, &Default::default()).await?;
    for s in &services.service {
      if !s.x_addr.starts_with(base_uri.as_str()) {
        return Err(format!(
          "Service URI {} is not within base URI {}",
          &s.x_addr, &base_uri
        ));
      }
      let url = Url::parse(&s.x_addr).map_err(|e| e.to_string())?;
      let svc = Some(
        soap::client::ClientBuilder::new(&url)
            .credentials(Some(creds.clone()))
            .build(),
      );
      match s.namespace.as_str() {
        "http://www.onvif.org/ver10/device/wsdl" => {
          if s.x_addr != devicemgmt_uri.as_str() {
            return Err(format!(
              "advertised device mgmt uri {} not expected {}",
              &s.x_addr, &devicemgmt_uri
            ));
          }
        }
        "http://www.onvif.org/ver10/events/wsdl" => {
          out.event = svc;
        }
        _ => {}
      }
    }
    if let Some(event) = &out.event {
      let mut duration = AbsoluteOrRelativeTimeType::Duration(Default::default());
      if let AbsoluteOrRelativeTimeType::Duration(d) = &mut duration {
        d.minutes = 1;
      }
      let consumer_reference = EndpointReferenceType {
        address: Default::default(), // Url::parse("http://192.168.1.118:8080/event").unwrap(),
        metadata: None,
        reference_parameters: None,
      };


      let req = schema::b_2::Subscribe {
        consumer_reference,
        filter: Default::default(),
        initial_termination_time: duration,
        subscription_policy: Default::default(),
      };


      println!("request: {}", yaserde::ser::to_string(&req).unwrap());

      let str_req = r#"<wsnt:Subscribe xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"><wsnt:ConsumerReference xmlns:tns="http://www.w3.org/2005/08/addressing">
      <tns:Address>http://192.168.1.90:8080/onvif/Notification</tns:Address></wsnt:ConsumerReference><wsnt:Filter /><wsnt:InitialTerminationTime>PT15M</wsnt:InitialTerminationTime><wsnt:SubscriptionPolicy /></wsnt:Subscribe>"#;

      // let str_req = r#"<wsnt:Subscribe xmlns:wsnt="http://docs.oasis-open.org/wsn/b-2"><wsnt:ConsumerReference xmlns:tns="http://www.w3.org/2005/08/addressing">
      // <tns:Address/></wsnt:ConsumerReference><wsnt:Filter /><wsnt:InitialTerminationTime>PT1M</wsnt:InitialTerminationTime><wsnt:SubscriptionPolicy /></wsnt:Subscribe>"#;


      // match schema::transport::request(event, &req).await {
      let resp = event.request(&str_req).await?;

      println!("response: {}", &resp);

      match yaserde::de::from_str(&resp) {
           Ok(schema::b_2::SubscribeResponse {
             subscription_reference, current_time, termination_time
           }) => println!("subscribed ref: {:#?} on: {:#?}, termination: {:#?}", subscription_reference, current_time, termination_time),
        Err(error) => println!("Failed to subscribe: {}", error.to_string()),
      }
    }

    /*
    got client: true
W20210630 10:17:48.118 s-front-main moonfire_base::clock] opening rtsp://192.168.1.110:554/h264Preview_01_main took PT1.019065148S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:03.671 s-front-main moonfire_base::clock] getting next packet took PT1.050840212S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3655", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:07.778 s-front-main moonfire_base::clock] getting next packet took PT1.573222930S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:13.530 s-front-main moonfire_base::clock] getting next packet took PT1.085234637S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3655", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:21.761 s-front-main moonfire_base::clock] getting next packet took PT1.033025463S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3655", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:36.257 s-front-main moonfire_base::clock] getting next packet took PT3.244742837S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:41.763 s-front-main moonfire_base::clock] getting next packet took PT1.130543727S!
W20210630 10:18:44.452 s-front-main moonfire_base::clock] getting next packet took PT1.031692276S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:50.075 s-front-main moonfire_base::clock] getting next packet took PT1.090896706S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
W20210630 10:18:55.547 s-front-main moonfire_base::clock] getting next packet took PT1.050492579S!
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3654", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
GOT NOTIFICATION: Request { method: POST, uri: /onvif/Notification, version: HTTP/1.1, headers: {"host": "192.168.1.90:8080", "user-agent": "gSOAP/2.8", "content-type": "application/soap+xml; charset=utf-8; action=\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\"", "content-length": "3655", "connection": "close", "soapaction": "\"http://docs.oasis-open.org/wsn/bw-2/NotificationConsumer/Notify\""}, body: Body(Streaming) }
I20210630 10:19:03.641 sync-/work/nvr moonfire_db::db] Flush 3 (why: 0 sec after start of 1 minute 14 seconds front-main recording 1/334):
/work/nvr: added 41M 955K 1020B in 1 recordings (1/334), deleted 40M 516K 94B in 1 (1/322), GCed 0 recordings ().

     */

    Ok(out)
  }
}

pub async fn process_notification(mut bytes: bytes::Bytes) {
  println!("process_notification {}", std::str::from_utf8(&bytes).unwrap());
}

// async fn get_capabilities(clients: &Clients) {
//   match schema::devicemgmt::get_capabilities(&clients.devicemgmt, &Default::default()).await {
//     Ok(capabilities) => println!("{:#?}", capabilities),
//     Err(error) => println!("Failed to fetch capabilities: {}", error.to_string()),
//   }
// }
//
// async fn get_device_information(clients: &Clients) {
//   println!(
//     "{:#?}",
//     &schema::devicemgmt::get_device_information(&clients.devicemgmt, &Default::default())
//         .await
//         .unwrap()
//   );
// }
//
// async fn get_service_capabilities(clients: &Clients) {
//   match schema::event::get_service_capabilities(&clients.devicemgmt, &Default::default()).await {
//     Ok(capability) => println!("devicemgmt: {:#?}", capability),
//     Err(error) => println!("Failed to fetch devicemgmt: {}", error.to_string()),
//   }
//
//   if let Some(ref event) = clients.event {
//     match schema::event::get_service_capabilities(event, &Default::default()).await {
//       Ok(capability) => println!("event: {:#?}", capability),
//       Err(error) => println!("Failed to fetch event: {}", error.to_string()),
//     }
//   }
//   if let Some(ref deviceio) = clients.deviceio {
//     match schema::event::get_service_capabilities(deviceio, &Default::default()).await {
//       Ok(capability) => println!("deviceio: {:#?}", capability),
//       Err(error) => println!("Failed to fetch deviceio: {}", error.to_string()),
//     }
//   }
//   if let Some(ref media) = clients.media {
//     match schema::event::get_service_capabilities(media, &Default::default()).await {
//       Ok(capability) => println!("media: {:#?}", capability),
//       Err(error) => println!("Failed to fetch media: {}", error.to_string()),
//     }
//   }
//   if let Some(ref media2) = clients.media2 {
//     match schema::event::get_service_capabilities(media2, &Default::default()).await {
//       Ok(capability) => println!("media2: {:#?}", capability),
//       Err(error) => println!("Failed to fetch media2: {}", error.to_string()),
//     }
//   }
//   if let Some(ref imaging) = clients.imaging {
//     match schema::event::get_service_capabilities(imaging, &Default::default()).await {
//       Ok(capability) => println!("imaging: {:#?}", capability),
//       Err(error) => println!("Failed to fetch imaging: {}", error.to_string()),
//     }
//   }
//   if let Some(ref ptz) = clients.ptz {
//     match schema::event::get_service_capabilities(ptz, &Default::default()).await {
//       Ok(capability) => println!("ptz: {:#?}", capability),
//       Err(error) => println!("Failed to fetch ptz: {}", error.to_string()),
//     }
//   }
//   if let Some(ref analytics) = clients.analytics {
//     match schema::event::get_service_capabilities(analytics, &Default::default()).await {
//       Ok(capability) => println!("analytics: {:#?}", capability),
//       Err(error) => println!("Failed to fetch analytics: {}", error.to_string()),
//     }
//   }
// }
//
// async fn get_system_date_and_time(clients: &Clients) {
//   let date =
//       schema::devicemgmt::get_system_date_and_time(&clients.devicemgmt, &Default::default())
//           .await;
//   println!("{:#?}", date);
// }
//
// async fn get_stream_uris(clients: &Clients) {
//   let media_client = clients.media.as_ref().unwrap();
//   let profiles = schema::media::get_profiles(media_client, &Default::default())
//       .await
//       .unwrap();
//   println!("get_profiles response: {:#?}", &profiles);
//   let requests: Vec<_> = profiles
//       .profiles
//       .iter()
//       .map(|p: &schema::onvif::Profile| schema::media::GetStreamUri {
//         profile_token: schema::onvif::ReferenceToken(p.token.0.clone()),
//         stream_setup: schema::onvif::StreamSetup {
//           stream: schema::onvif::StreamType::RtpUnicast,
//           transport: schema::onvif::Transport {
//             protocol: schema::onvif::TransportProtocol::Rtsp,
//             tunnel: vec![],
//           },
//         },
//       })
//       .collect();
//
//   let responses = futures_util::future::try_join_all(
//     requests
//         .iter()
//         .map(|r| schema::media::get_stream_uri(media_client, r)),
//   )
//       .await
//       .unwrap();
//   for (p, resp) in profiles.profiles.iter().zip(responses.iter()) {
//     println!("token={} name={}", &p.token.0, &p.name.0);
//     println!("    {}", &resp.media_uri.uri);
//     if let Some(ref v) = p.video_encoder_configuration {
//       println!(
//         "    {:?}, {}x{}",
//         v.encoding, v.resolution.width, v.resolution.height
//       );
//       if let Some(ref r) = v.rate_control {
//         println!("    {} fps, {} kbps", r.frame_rate_limit, r.bitrate_limit);
//       }
//     }
//     if let Some(ref a) = p.audio_encoder_configuration {
//       println!(
//         "    audio: {:?}, {} kbps, {} kHz",
//         a.encoding, a.bitrate, a.sample_rate
//       );
//     }
//   }
// }
//
// async fn get_hostname(clients: &Clients) {
//   let resp = schema::devicemgmt::get_hostname(&clients.devicemgmt, &Default::default())
//       .await
//       .unwrap();
//   debug!("get_hostname response: {:#?}", &resp);
//   println!(
//     "{}",
//     match resp.hostname_information.name {
//       Some(ref h) => &h,
//       None => "(unset)",
//     }
//   );
// }
//
// async fn set_hostname(clients: &Clients, hostname: String) {
//   schema::devicemgmt::set_hostname(
//     &clients.devicemgmt,
//     &schema::devicemgmt::SetHostname { name: hostname },
//   )
//       .await
//       .unwrap();
// }
//
// async fn enable_analytics(clients: &Clients) {
//   let media_client = clients.media.as_ref().unwrap();
//   let mut config = schema::media::get_metadata_configurations(media_client, &Default::default())
//       .await
//       .unwrap();
//   if config.configurations.len() != 1 {
//     println!("Expected exactly one analytics config");
//     return;
//   }
//   let mut c = config.configurations.pop().unwrap();
//   let token_str = c.token.0.clone();
//   println!("{:#?}", &c);
//   if c.analytics != Some(true) || c.events.is_none() {
//     println!(
//       "Enabling analytics in metadata configuration {}",
//       &token_str
//     );
//     c.analytics = Some(true);
//     c.events = Some(schema::onvif::EventSubscription {
//       filter: None,
//       subscription_policy: None,
//     });
//     schema::media::set_metadata_configuration(
//       media_client,
//       &schema::media::SetMetadataConfiguration {
//         configuration: c,
//         force_persistence: true,
//       },
//     )
//         .await
//         .unwrap();
//   } else {
//     println!(
//       "Analytics already enabled in metadata configuration {}",
//       &token_str
//     );
//   }
//
//   let profiles = schema::media::get_profiles(media_client, &Default::default())
//       .await
//       .unwrap();
//   let requests: Vec<_> = profiles
//       .profiles
//       .iter()
//       .filter_map(
//         |p: &schema::onvif::Profile| match p.metadata_configuration {
//           Some(_) => None,
//           None => Some(schema::media::AddMetadataConfiguration {
//             profile_token: schema::onvif::ReferenceToken(p.token.0.clone()),
//             configuration_token: schema::onvif::ReferenceToken(token_str.clone()),
//           }),
//         },
//       )
//       .collect();
//   if !requests.is_empty() {
//     println!(
//       "Enabling metadata on {}/{} configs",
//       requests.len(),
//       profiles.profiles.len()
//     );
//     futures_util::future::try_join_all(
//       requests
//           .iter()
//           .map(|r| schema::media::add_metadata_configuration(media_client, r)),
//     )
//         .await
//         .unwrap();
//   } else {
//     println!(
//       "Metadata already enabled on {} configs",
//       profiles.profiles.len()
//     );
//   }
// }
//
// async fn get_analytics(clients: &Clients) {
//   let config = schema::media::get_video_analytics_configurations(
//     clients.media.as_ref().unwrap(),
//     &Default::default(),
//   )
//       .await
//       .unwrap();
//   println!("{:#?}", &config);
//   let c = match config.configurations.first() {
//     Some(c) => c,
//     None => return,
//   };
//   if let Some(ref a) = clients.analytics {
//     let mods = schema::analytics::get_supported_analytics_modules(
//       a,
//       &schema::analytics::GetSupportedAnalyticsModules {
//         configuration_token: schema::onvif::ReferenceToken(c.token.0.clone()),
//       },
//     )
//         .await
//         .unwrap();
//     println!("{:#?}", &mods);
//   }
// }
//
// async fn get_status(clients: &Clients) {
//   if let Some(ref ptz) = clients.ptz {
//     let media_client = clients.media.as_ref().unwrap();
//     let profile = &schema::media::get_profiles(media_client, &Default::default())
//         .await
//         .unwrap()
//         .profiles[0];
//     let profile_token = schema::onvif::ReferenceToken(profile.token.0.clone());
//     println!(
//       "ptz status: {:#?}",
//       &schema::ptz::get_status(ptz, &schema::ptz::GetStatus { profile_token })
//           .await
//           .unwrap()
//     );
//   }
// }
//
// #[tokio::main]
// async fn main() {
//   env_logger::init();
//
//   let args = Args::from_args();
//   let clients = Clients::new(&args).await.unwrap();
//
//   match args.cmd {
//     Cmd::GetSystemDateAndTime => get_system_date_and_time(&clients).await,
//     Cmd::GetCapabilities => get_capabilities(&clients).await,
//     Cmd::GetServiceCapabilities => get_service_capabilities(&clients).await,
//     Cmd::GetStreamUris => get_stream_uris(&clients).await,
//     Cmd::GetHostname => get_hostname(&clients).await,
//     Cmd::SetHostname { hostname } => set_hostname(&clients, hostname).await,
//     Cmd::GetDeviceInformation => get_device_information(&clients).await,
//     Cmd::EnableAnalytics => enable_analytics(&clients).await,
//     Cmd::GetAnalytics => get_analytics(&clients).await,
//     Cmd::GetStatus => get_status(&clients).await,
//     Cmd::GetAll => {
//       get_system_date_and_time(&clients).await;
//       get_capabilities(&clients).await;
//       get_service_capabilities(&clients).await;
//       get_stream_uris(&clients).await;
//       get_hostname(&clients).await;
//       get_analytics(&clients).await;
//       get_status(&clients).await;
//     }
//   }
// }