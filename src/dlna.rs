use crate::devices::Render;
use crate::streaming::MediaStreamingServer;
use xml::escape::escape_str_attribute;

const PAYLOAD_PLAY: &str = r#"
    <InstanceID>0</InstanceID>
    <Speed>1</Speed>
"#;

pub async fn play(render: Render, streaming_server: MediaStreamingServer) {

    let subtitle_uri = streaming_server.subtitle_uri();
    let payload_subtitle = match subtitle_uri {
        Some(subtitle_uri) => {
            escape_str_attribute(
                format!(r###"
                    <DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
                               xmlns:dc="http://purl.org/dc/elements/1.1/" 
                               xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/" 
                               xmlns:dlna="urn:schemas-dlna-org:metadata-1-0/" 
                               xmlns:sec="http://www.sec.co.kr/" 
                               xmlns:xbmc="urn:schemas-xbmc-org:metadata-1-0/">
                        <item id="0" parentID="-1" restricted="1">
                            <dc:title>nano-dlna Video</dc:title>
                            <res protocolInfo="http-get:*:video/{type_video}:" xmlns:pv="http://www.pv.com/pvns/" pv:subtitleFileUri="{uri_sub}" pv:subtitleFileType="{type_sub}">{uri_video}</res>
                            <res protocolInfo="http-get:*:text/srt:*">{uri_sub}</res>
                            <res protocolInfo="http-get:*:smi/caption:*">{uri_sub}</res>
                            <sec:CaptionInfoEx sec:type="{type_sub}">{uri_sub}</sec:CaptionInfoEx>
                            <sec:CaptionInfo sec:type="{type_sub}">{uri_sub}</sec:CaptionInfo>
                            <upnp:class>object.item.videoItem.movie</upnp:class>
                        </item>
                    </DIDL-Lite>
                    "###,
                    uri_video = streaming_server.video_uri(),
                    type_video = streaming_server.video_type(), 
                    uri_sub = subtitle_uri,
                    type_sub = streaming_server.subtitle_type().unwrap_or("unknown".to_string())
                ).as_str()
           ).to_string()
        }
        None => "".to_string()
    };

    let payload_setavtransporturi = format!(r#"
        <InstanceID>0</InstanceID>
        <CurrentURI>{}</CurrentURI>
        <CurrentURIMetaData>{}</CurrentURIMetaData>
        "#,
        streaming_server.video_uri(),
        payload_subtitle
    );

    println!("Starting media streaming server...");
    let streaming_server_handle = tokio::spawn(async move {	
        streaming_server.run().await
    });

    println!("Setting Video URI");
    render.service.action(
        render.device.url(), 
        "SetAVTransportURI", 
        payload_setavtransporturi.as_str()
    )
    .await
    .unwrap_or_else(
        |e| panic!("Unable to SetAVTransportURI, error: {}", e)
    );

    println!("Playing video");
    render.service.action(
        render.device.url(), 
        "Play", 
        PAYLOAD_PLAY
    )
    .await
    .unwrap_or_else(
        |e| panic!("Unable to Play, error: {}", e)
    );

    streaming_server_handle
        .await
        .unwrap_or_else(
            |e| panic!("Error while serving the media files: {}", e)
        );
}