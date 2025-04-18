<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:xs="http://www.w3.org/2001/XMLSchema"
    exclude-result-prefixes="xs"
    version="3.0">
    <xsl:output method="xml" indent="no"/>
    <xsl:strip-space elements="*"/>
    <xsl:template match="/">
        
        <events>
            <xsl:merge>
                <xsl:merge-source for-each-source="'log-file-1.xml'" select="events/event" streamable="yes">
                        <xsl:merge-key select="xs:dateTime(@timestamp)"/>
                </xsl:merge-source>
                <xsl:merge-source for-each-source="'log-file-2.xml'" select="log/*/record" streamable="yes" sort-before-merge="yes">
                        <xsl:merge-key select="dateTime(../@date, time)"/>
                </xsl:merge-source>
                <xsl:merge-action>
                    <group>
                        <xsl:copy-of select="current-merge-group()" />
                    </group>
                </xsl:merge-action>
            </xsl:merge>
        </events>
        
    </xsl:template>
</xsl:stylesheet>